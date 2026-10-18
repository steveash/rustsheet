---
id: T-1817
title: "Probe REGEXREPLACE results past 32,767 characters"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's result for a REGEXREPLACE result past the 32,767-character text limit is probed and ours matches it.

## Context
T-1272 (performance and security reviews) already sizes the result before building it: `Replacement::result_len` in
functions/text/regex_functions.rs, and a result over 32,767 UTF-16 units is `#VALUE!`, as SUBSTITUTE's is
(text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped", UNVERIFIED). Use the excel-probe skill:
`=LEN(REGEXREPLACE(REPT("a",32767),"","bbbbbbbbbb"))`, `=LEN(REGEXREPLACE(REPT("a",32767),"a","bb"))`, a result of
exactly 32,767 and 32,768 units, and one with an emoji straddling the limit. If Excel truncates or gives another error,
change `too_long` in regex_functions.rs to match. Also probe the replacement syntax ours takes from the regex crate
(text.md, same entry): `REGEXREPLACE("ab","(a)(b)","[$1a|$|$$|${1}]")` (ours `[|$|$|a]`) and a named group
`REGEXREPLACE("ab","(?<x>a)","${x}$x")`; a syntax fix must keep `Replacement` (the size check) in step.

- From T-1272's fidelity review: whether `\w`, `\b` and `\pL` are Unicode in Excel (`REGEXTEST("é","\w")`,
  `REGEXEXTRACT("naïve","\w+")`, `REGEXTEST("é","\pL")`); ours follow the regex crate (Unicode).

## Acceptance criteria
- [ ] A probe case with oracle values covers the cells above; text.md marks the rule VERIFIED or records Excel's.
- [ ] Ours matches; `make check` green; reviewers approve.

## Out of scope
- The bound itself (T-1272).

## Handoff (resumed)
Done: probes/regexreplace-limit-and-syntax (oracle filled; findings in text.md "REGEXREPLACE results are not capped at 32,767 characters"). Excel has no 32,767 cap inside a formula (65,534-unit result works); a 360,447-unit result is `#CALC!`; excel-graph refuses probe workbooks that cache a large result, so the real cap cannot be bisected on the web. Mismatches left (5 cells, now in the baseline): A1 (`#CALC!` vs our `#VALUE!`), A2, A5, A6, A9 (`too_long` in functions/text/regex_functions.rs rejects results Excel allows) and A23 (`[[:alpha:]]` is ASCII-only in ours, Unicode in Excel). Next: choose our cap for large results as a directive-5 DEVIATION (with a decision record and a `DEVIATION` entry), return `#CALC!` above it, keep `Replacement::result_len` in step, make POSIX classes Unicode, and re-fill A17/A18 once the oracle decodes the rich error. Needs a design call on the cap, so opus.
