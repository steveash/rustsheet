---
id: T-1829
title: "REGEX* match iteration on the PikeVM fallback is charged by measured work, not the rest of the text"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1272]
---

## Goal
A REGEX* call that iterates matches (REGEXEXTRACT with return_all, REGEXREPLACE) on the PikeVM fallback is charged
about the work it does, so ordinary `\b` patterns over non-ASCII text of a few KB return values again, as on main
before T-1272 and in Excel, while every search stays bounded (CLAUDE.md directive 5).

## Context
T-1272's final performance review. `Search::find_in` (functions/text/regex_engine.rs) charges a PikeVM search its
whole span times the NFA's state count before it runs. In `for_each_match` and `for_each_captures`, each successive
search's span is the rest of the text, so m matches in n bytes cost about m * n / 2 * states. The lazy DFA quits on
the first non-ASCII byte when the pattern has a Unicode `\b` (`unicode_word_boundary(true)`), so such patterns run on
the PikeVM. Each of these is `#ERROR!` after under 1 ms of real work:
- `COLUMNS(REGEXEXTRACT(REPT("naïve ",500),"\b\w+\b",1))` (300 repeats returns 300)
- `LEN(REGEXREPLACE(REPT("café foo bar ",250),"\b(\w+)\b","<$1>"))`
- `LEN(REGEXREPLACE(REPT("café foo bar ",2500),"\bfoo\b","x"))` (`cafe` returns 27500)
They are pinned by `pike_iteration_is_charged_by_rest_of_text` in test/text_functions/fn_regex.rs and recorded in
text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped".
A true quadratic case exists (`a.*z|a` over `aaa...` with no `z`: each search scans to the end), so any deduction must
stay sound for it. Options:
- measure how far the PikeVM scanned, for example with a small instrumented PikeVM, or with a bounded backtracker over
  doubling windows that can tell when a window was enough;
- or avoid the fallback for Unicode `\b`, if Excel's PCRE2 treats `\b`/`\w` as ASCII. That is UNVERIFIED: probe it
  with the `excel-probe` skill first.

## Acceptance criteria
- [x] The three formulas above return values (the test is updated to expect them) and stay fast in a release build.
- [x] `a.*z|a` iteration over a long `aaa...` text is still bounded (`#ERROR!` within about 1 s).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A PikeVM search (the fallback when the lazy DFA quits, e.g. on a Unicode `\b` over non-ASCII text) now runs over the
span a set-based scan of the NFA shows a leftmost-first search needs (`regex_reach.rs`), charged 21 units for each state
the scan visits, instead of being charged its whole span up front. The three formulas return 500, 27500 and 4750
(`pike_iteration_is_charged_by_work_done`); `a.*z|a|\bq` over `REPT("a",20000)&"é"` is still `#ERROR!`, about 1 s in
release (`quadratic_pike_iteration_is_still_bounded`); `pike_search_span_keeps_leftmost_first_matches` pins that the
shortened span keeps leftmost-first results. The probe probes/regex-ascii-classes (42 cells) shows Excel's `\w`, `\b`,
`\d`, `\s` are Unicode (UCP), so the ASCII shortcut the task offered is not Excel's; recorded in text.md "REGEX*
character classes are Unicode", with its 7 matcher gaps added to T-1900. Eval: no regressions; the new case is 35/42.

## Remaining (session budget ran out; PR #1153 is a draft, resume from this branch)
Reviewers: security, architecture, quality and Excel fidelity APPROVE; performance REQUEST_CHANGES. To do:
- MAJOR: charge `State::Look` visits more in `Reach::close` (a `LOOK_VISIT` surcharge, about 4x a plain visit, or
  only on `WordUnicode*`). The reviewer measured these in release, with 21 units a visit:
  - `\b` x40 over `REPT("𝒜 ",8000)&SEQUENCE(30)` takes 3.9 s to exhaust the budget.
  - `\B` x40 & "q" over `REPT("éa ",10000)&SEQUENCE(300)` takes 3.55 s.
  - `(?:\b|\B){40}q` over the same text: main gives 0 in 977 ms, the branch `#ERROR!` after 1.7 s.
  Calibrate on a `\b`/`\B` chain over 4-byte characters, aim for about 1 s, and document it next to `SCAN_VISIT`.
- MINOR: single PikeVM searches now pay for the scan plus the PikeVM (`REGEXTEST(REPT("é ",15000),"\bzz\b")`
  went from 2.0 ms to 3.5 ms). Either justify this, or skip the scan when the old up-front charge (span x states x 5)
  is no more than the scan's likely cost (`earliest` searches, the first search).
- MINOR: add two cases to the soundness doc in regex_reach.rs:
  - the PikeVM's empty-match retries (`skip_splits_fwd`) restart at or before the scan's first boundary match, so
    their state sets are subsets of the scan's;
  - always-anchored patterns, where the scan overcharges.
- Follow-up (file with `make new-task`): add states to the closure in priority order and cut lower-priority threads
  when one matches, as the PikeVM's `nexts` does. Lazy patterns stay quadratic until then:
  `COLUMNS(REGEXEXTRACT(REPT("é ab ",4000),"\bab.*?\b",1))` is `#ERROR!`, as on main.
