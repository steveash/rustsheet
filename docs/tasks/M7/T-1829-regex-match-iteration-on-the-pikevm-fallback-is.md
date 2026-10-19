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
- [ ] The three formulas above return values (the test is updated to expect them) and stay fast in a release build.
- [ ] `a.*z|a` iteration over a long `aaa...` text is still bounded (`#ERROR!` within about 1 s).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
