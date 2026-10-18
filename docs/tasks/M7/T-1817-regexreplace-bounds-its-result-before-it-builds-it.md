---
id: T-1817
title: "Probe REGEXREPLACE results past 32,767 characters"
milestone: M7
lane: functions-text
status: done
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
- [x] A probe case with oracle values covers the cells above; text.md marks the rule VERIFIED or records Excel's.
- [x] Ours matches; `make check` green; reviewers approve.

## Out of scope
- The bound itself (T-1272).

## Handoff (resumed)
Done: probes/regexreplace-limit-and-syntax (oracle filled; findings in text.md "REGEXREPLACE results are capped at 65,535 UTF-16 units" (heading renamed on resume)). Excel has no 32,767 cap inside a formula (65,534-unit result works); a 360,447-unit result is `#CALC!`; excel-graph refuses probe workbooks that cache a large result, so the real cap cannot be bisected on the web. Mismatches left (5 cells, now in the baseline): A1 (`#CALC!` vs our `#VALUE!`), A2, A5, A6, A9 (`too_long` in functions/text/regex_functions.rs rejects results Excel allows) and A23 (`[[:alpha:]]` is ASCII-only in ours, Unicode in Excel). Next: choose our cap for large results as a directive-5 DEVIATION (with a decision record and a `DEVIATION` entry), return `#CALC!` above it, keep `Replacement::result_len` in step, make POSIX classes Unicode, and re-fill A17/A18 once the oracle decodes the rich error. Needs a design call on the cap, so opus.

## Outcome
Excel caps a REGEXREPLACE result at 65,535 UTF-16 units and is `#CALC!` past it (bisected with
probes/regexreplace-result-cap: 65,535 passes, 65,536 does not, emoji counting two), so no DEVIATION or decision
record was needed: `MAX_REPLACED_UNITS` / `over_cap` replace the 32,767 `#VALUE!` check, still applied to the length
measured before the result is built; the lifted broadcast budget stays `#VALUE!`. The probes also showed that TYPE is
64 and ERROR.TYPE 3 for any text over 255 units (probes/type-long-text), now implemented with a `MAX_SHORT_TEXT_UNITS`
constant shared with DECIMAL. Downstream behavior on over-32,767 text is recorded in text.md
(probes/regexreplace-long-result-a to -g, -i). Eval: 5 handoff cells fixed (regexreplace-limit-and-syntax A1, A2, A5,
A6, A9) plus 66 of 69 new probe cells passing; 3 left baselined with follow-ups: T-1902 (SUBSTITUTE, UPPER of
over-limit text) and T-1903 (POSIX classes Unicode, A23). A17/A18 stay pending until the oracle decodes rich errors.
