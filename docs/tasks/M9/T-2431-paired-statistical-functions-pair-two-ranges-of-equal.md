---
id: T-2431
title: "Paired statistical functions pair two ranges of equal count but different shape instead of #N/A (gen-m9, 17 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
CORREL, INTERCEPT, FORECAST, FORECAST.LINEAR, CHISQ.TEST and CHITEST pair two arrays with the same number of cells and
different shapes, as Excel does, instead of `#N/A`; only a different count is `#N/A`.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "Paired functions pair two ranges of equal count whatever their shapes
(T-2255)" records it: `CORREL(Data!B11:E12,Data!D9:E12)` is 1, `INTERCEPT(Data!E5:E8,Data!B3:C4)` -0.15000001236262528,
`CHITEST(Data!D10:E12,Data!A9:C10)` 0. 17 cells; the pairing order (row by row in each array) is the natural reading, and
each cell's Excel value checks it. One cell, `INTERCEPT((OFFSET(..)*VLOOKUP(..)),Data!A6:A9)` (a61e547a1cab994c A382), is
`#VALUE!` in Excel: its known_y is an array holding an error; keep it if the pairing fix does not give it, and say so.
Code: `fn_get_two_matrices` (functions/math_and_trigonometry/sum.rs) and the CHISQ.TEST reader (functions/statistical/
chisq.rs): compare cell counts, not rows and columns.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 formulas.
Examples:
- gen-m9/0a9226016e8f5f33 Calc!A291 `=CHITEST(Data!D10:E12,Data!A9:C10)`: Excel 0.0, ours #N/A
- gen-m9/0a9226016e8f5f33 Calc!A333 `=CHISQ.TEST(Data!A10:B12,Data!A8:C9)`: Excel #DIV/0!, ours #N/A
- gen-m9/a61e547a1cab994c Calc!A382 `=INTERCEPT((OFFSET(Data!A6:B12,0,2,2,2)*VLOOKUP(3.14159,Data!C11:C12,1,Data!C12)),Data!A6:A9)`: Excel #VALUE!, ours #N/A
Cells:
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A291, A333
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A370
- gen-m9/530c636749a85209.case.json (1): Calc!A388
- gen-m9/7674c89c824aca43.case.json (1): Calc!A72
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A38, A263
- gen-m9/a54c738c697617c3.case.json (1): Calc!A291
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A156
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A382
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A185
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A32, A169
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A94, A276, A314
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A43

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 17 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `CORREL({1,2;3,4},{1;2;3;4})` and `INTERCEPT({1;2;3;4},{1,2;3,5})` give the paired values.
- [ ] statistics.md's entry gets an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Different counts (unchanged `#N/A`).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
