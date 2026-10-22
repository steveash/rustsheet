---
id: T-2380
title: "SERIESSUM: a negative x to a non-integer power does not stop the read; a later text coefficient is #VALUE!"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
SERIESSUM with a negative x and a non-integer power gives Excel's result when a later coefficient is text: `#VALUE!` rather than an immediate `#NUM!`.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `SERIESSUM(-308641972.8075,30864197.28075,2,Data!D1:E8)` is `#VALUE!` in Excel; ours is `#NUM!`. D1:E8 read row by row is 2,
1E-15, the text "#N/A", the text "ü", ..., so the first term is `2*x^n` with x < 0 and n not an integer, which `^` refuses with
`#NUM!`, and T-2163's `fn_seriessum` (each term through `arithmetic::power`, `#NUM!` at once when the power fails) stops there.
Excel goes on to the text coefficient. An overflowing term, by contrast, is `#NUM!` before a later text cell (numbers.md "SERIESSUM:
blank coefficient cells are dropped; an overflowing term beats a text coefficient"). Observation: numbers.md "SERIESSUM: a negative
x to a non-integer power does not stop the read" (added by this triage).
Needs a probe first (`excel-probe` skill, excel-graph): `SERIESSUM(-2,0.5,1,{1,"a"})`, `(-2,0.5,1,{1,2})`, `(-2,0.5,1,{1,NA()})`,
`(-2,0.5,1,{"a",1})`, `(-2,0.5,2,{1,1})` (powers 0.5, 2.5), `(-2,1,0.5,{1,1,"a"})` (second term bad), `(-2,0.5,1,{0,"a"})`
(zero coefficient), the same with a range of cells, and an overflowing then a domain-failing term before text.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/bf8903e05ec369ca Calc!A326 | `=SERIESSUM(SUM(((-2.5)*123456789.123),((-1)+Data!B2)),(123456789.123*(0.5^2)),2,Data!D1:E8)` | #VALUE! | #NUM! |

## Acceptance criteria
- [x] A probes case with the cells above, oracle-filled (excel-graph), passes in full; numbers.md states the rule.
- [x] Every SERIESSUM cell of probes/seriessum-* and gen-m5 that passes today still passes.
- [x] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- SERIESSUM with x = 0 (T-1359, done).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
SERIESSUM counts a term with a negative x and a non-integer power as 0 and reads on (`seriessum_of`); overflow stays an
immediate `#NUM!`. Probe probes/seriessum-negative-x-fractional-power (21 cells, excel-graph) pins it: `{1,2}` is 0,
`(-2,1,0.5,{1,1,1})` is 2, a later text/boolean is `#VALUE!`, a later error that error. The old unit test that pinned `#NUM!`
(no evidence) now asserts Excel's 0. numbers.md entry is VERIFIED.
Eval: gen-m5 known failures down by 1 (bf8903e05ec369ca A326); probes +1 case (21 cells, all pass); no regressions.
