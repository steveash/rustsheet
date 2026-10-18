---
id: T-1904
title: "SUBSTITUTE and UPPER of text over 32,767 units are #VALUE!"
milestone: M2
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBSTITUTE and UPPER of text over 32,767 UTF-16 units are `#VALUE!`, as Excel's are.

## Context
Since T-1817 REGEXREPLACE returns results up to 65,535 units, so over-limit text reaches other functions. Excel
(probes/regexreplace-long-result-d A2, -e A2, excel-graph): `SUBSTITUTE(REGEXREPLACE(REPT("a",32767),"a","bb"),"b","c")`
and `UPPER(REGEXREPLACE(REPT("a",32767),"a","bb"))` are `#VALUE!`; ours return the 65,534-unit text (`LEN` 65534).
`substitute_exceeds_limit` in functions/text/util.rs (and the UTF-16 path near `result_len > MAX_TEXT_UNITS`) passes
an over-limit input through when the result does not grow, which this evidence contradicts (text.md "SUBSTITUTE result
longer than 32,767 characters", Open). Excel takes the same text as text in `LEN`, `LEFT`, `MID`, `ISTEXT` and the
REGEX* functions (text.md "REGEXREPLACE results are capped at 65,535 UTF-16 units"), so the rule is per function, not
a general coercion rule. Probe LOWER, PROPER, TRIM, CLEAN, REPLACE and `&` on the same input with the excel-probe skill
first (at most two long texts per case: excel-graph refuses a workbook that builds more, see that entry's probing note)
and fix every function the probe shows rejecting it.

## Acceptance criteria
- [ ] probes/regexreplace-long-result-d A2 and -e A2 pass; any new probe cells pass or are baselined with a follow-up.
- [ ] text.md records the rule per function probed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- REGEXREPLACE's own cap (T-1817).

## Outcome
SUBSTITUTE, UPPER, LOWER, PROPER, TRIM, CLEAN and REPLACE return `#VALUE!` for text over 32,767 UTF-16 units
(shared `reject_over_limit` in functions/text/common.rs). New probes text-long-input-a/-b/-c (excel-graph) show
all of them `#VALUE!`; `&` is unprobed (excel-graph refuses the workbook). probes/regexreplace-long-result-d and
-e A2 now pass (known failures removed from eval/baseline/probes.txt). Rule recorded in text.md.
