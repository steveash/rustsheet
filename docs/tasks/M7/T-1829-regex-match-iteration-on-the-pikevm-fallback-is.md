---
id: T-1829
title: "REGEX* match iteration on the PikeVM fallback is charged by measured work, not the rest of the text"
milestone: M7
lane: functions-text
status: done
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
- [x] `make check` green; reviewers approve.

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

## Review round 2 (resumed by a later opus run)
The performance review's findings from the first round:
- MAJOR, done: `Reach::close` counts the look-around assertions it evaluates, and each costs `LOOK_VISIT` (54
  units) on top of its 21-unit visit, calibrated in release on `\B` over `é` (the worst measured: about 54 ns
  more than a visit; 49 ns for `\b` over `𝒜`, 27 ns for `(?:\b|\B){40}q`). Release, time to exhaust the
  budget, before -> after: `REPT("\B",40)&"q"` over `REPT("éa ",10000)&SEQUENCE(300)` 3.4 s -> 0.97 s;
  `(?:\b|\B){40}q` over it 1.5 s -> 0.81 s; `REPT("\b",40)&"q"` over `REPT("𝒜 ",8000)&SEQUENCE(30)` returned
  0 after 1.35 s, now `#ERROR!` after 0.94 s (the cap of text.md, about 1 s). The task's three formulas are
  unchanged (1.0, 1.5 and 4.8 ms).
- MINOR, kept and justified: a single `PikeVM` search pays for the scan plus the `PikeVM` (about 1.6x the old real
  time: `REGEXTEST(REPT("é ",15000),"\bzz\b")` 3.1 ms). The scan is what makes the charge a measurement. Skipping
  it when the old up-front charge (span x states x 5) looks cheap would bring back a guess that undercharged
  look-around chains and overcharged iteration, and the extra cost is linear and well under the 1 s cap.
- MINOR, done: the module doc of regex_reach.rs now covers the empty-match retries (`skip_splits_fwd`) and
  always-anchored patterns (overcharged, linearly).
- Follow-ups filed: T-1985 (states in priority order, so lazy patterns are not quadratic); T-1988 (round 2 MINOR:
  charge a `Sparse` state visit by the transitions it walks; `\b[\w\s\d]{30}q` over CJK text exhausts in about 1.7 s).
