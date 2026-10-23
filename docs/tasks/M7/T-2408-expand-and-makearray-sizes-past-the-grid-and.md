---
id: T-2408
title: "EXPAND and MAKEARRAY sizes past the grid, and MAKEARRAY argument errors before size validation (gen-m7)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
EXPAND and MAKEARRAY give Excel's error for sizes past the grid: EXPAND rows of about 1E+21 or more is `#VALUE!` (not `#NUM!`),
EXPAND with 123,456,789 columns is `#NUM!` (not our `#ERROR!` cell cap), MAKEARRAY with 25,000 columns is built and then `#SPILL!`
(not `#VALUE!`), and MAKEARRAY returns an error in its columns before it rejects a fractional rows.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 5 cells, 5 formulas, one cell each:
- `EXPAND(1e-15,ROUND(1E+21,45),1)` is `#VALUE!` (2d9b48c6363e7a16 A2454) and `EXPAND(Data!A11:A12,(7*Data!C4)^WEEKNUM(44793,1))`, rows
  (-17.5)^34 = 1.83E+42, is `#VALUE!` (8c4570978f18f4c2 A4006); ours `#NUM!` for both. arrays.md "Stacked results past the grid" has
  `ROWS(EXPAND(1,1048577,1))` and `EXPAND(1,1E9,1E9)` as `#NUM!` (probes/expand-wrap-past-grid), so the error changes somewhere between
  1E+9 and 1E+21 rows; 2^31, where WRAPROWS's wrap_count turns `#NUM!` (T-2312), is a guess to probe, not a rule.
- `EXPAND(LOWER("a"),4,Data!B$10)` with B10 = 123456789.123 is `#NUM!` (4dac39f89709e2fc A2150); ours `#ERROR!`, the 2^20-cell allocation
  bound (arrays.md "Array results above 1,048,576 cells are #ERROR!"). Excel refuses this one itself, so `#NUM!` here is parity, not a
  DEVIATION; find where Excel's columns limit for EXPAND lies (16,385 columns are allowed: `COLUMNS(EXPAND(1,1,16385))`).
- `MAKEARRAY(4,25000,_xleta.AVERAGE)` (the 25000 is `LAMBDA(v,x,x)(Data!A9,Data!D10)/(-0.0001)`, exactly 25000) is `#SPILL!`
  (21b4eef18424ce94 A3348): Excel builds 4 x 25,000 cells, wider than the grid, and cannot spill them. Ours `#VALUE!`
  (`MAKEARRAY(2,20000,..)` too), so our MAKEARRAY refuses columns past 16,384 where HSTACK, EXPAND and WRAPROWS allow them
  (`limits::check_stack_size`). The 100,000 cells are under our 2^20-cell bound.
- `MAKEARRAY(SUM(..)=123456794.36459264,SUM(..)*ACOS(1+2^-52),..)` is `#NUM!` (bc4a69b0b512bc04 A1028): the columns' `#NUM!` (ACOS
  out of domain) comes before the fractional rows' `#VALUE!` (arrays.md "MAKEARRAY rows and columns must be whole numbers"). Ours `#VALUE!`
  (`MAKEARRAY(1.5,NA(),..)` is `#VALUE!` too). Read both sizes, errors first, then validate, as SEQUENCE does (arrays.md "SEQUENCE and
  TAKE: an argument error before zero rows").
Probe first (excel-probe skill): EXPAND rows 2^31-1, 2^31, 1E+10, 1E+15, a negative rows, columns 16,385 x 4 rows, 1E+6, 2^24, 2^31 with
1 and 4 rows; MAKEARRAY columns 16,385 and 1E+6 (ROWS and COLUMNS of it, and bare), rows 1,048,577, and `MAKEARRAY(1.5,NA(),..)`,
`MAKEARRAY(NA(),1.5,..)`. Code: `fn_expand` (functions/lookup_and_reference/expand.rs), `fn_makearray`/`makearray_kernel`
(functions/logical/bycol_byrow.rs), limits.rs. Quirk entries: arrays.md "Stacked results past the grid" and "MAKEARRAY rows and columns
must be whole numbers" (sentences added by this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 5 known-failure lines in eval/baseline/gen-m7.txt, 5 formulas.
- gen-m7/21b4eef18424ce94.case.json A3348 `=MAKEARRAY(4,(LAMBDA(v,x,x)(Data!A9,Data!D10)/(-0.0001)),_xleta.AVERAGE)`: A3348 Excel #SPILL!, ours #VALUE!; cells (1): A3348
- gen-m7/2d9b48c6363e7a16.case.json A2454 `=EXPAND(1e-15,ROUND(1E+21,(3.14159+42)),1)`: A2454 Excel #VALUE!, ours #NUM!; cells (1): A2454
- gen-m7/4dac39f89709e2fc.case.json A2150 `=EXPAND(LOWER("a"),4,Data!B$10)`: A2150 Excel #NUM!, ours #ERROR!; cells (1): A2150
- gen-m7/8c4570978f18f4c2.case.json A4006 `=EXPAND(Data!A11:A12,((7*Data!C4)^WEEKNUM(44793,1)))`: A4006 Excel #VALUE!, ours #NUM!; cells (1): A4006
- gen-m7/bc4a69b0b512bc04.case.json A1028 `=MAKEARRAY(SUM(((-0.5)*0),Data!C2:C6,PI()),(SUM(Data!D2:D5)*ACOS(1+2^-52)),_xleta.AVERAGE)`: A1028 Excel #NUM!, ours #VALUE!; cells (1): A1028

## Acceptance criteria
- [x] A probe case with the cells above, filled from excel-graph; arrays.md states the thresholds it shows.
- [x] Visible parity: these 5 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline; every probe cell passes or has its own task.
- [x] Any size Excel builds but ours refuses for memory stays a documented DEVIATION; none is added for a size Excel itself refuses.
- [x] `make check` green; reviewers approve.

## Out of scope
- EXPAND's pad_with (T-2415); MAKEARRAY sizes within an ulp of a whole number (T-2345).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probes expand-makearray-past-grid (29 cells) and -big (6 cells), excel-graph 2026-10-23: EXPAND rows or columns of 2^31 or more
are `#VALUE!`, rows below 2^31 past the grid `#NUM!`; columns have an Excel cap of their own somewhere in 16,386..123,456,789
(`#NUM!` even in one row), checked after the smaller-than-the-array `#VALUE!`. MAKEARRAY reads both sizes (errors first) before
validating them, and builds columns past the grid under the cell bound (above it, `#VALUE!` as before). Larger builds hang
excel-graph's workbook open, so the exact column cap is T-2452. Eval: gen-m7 known failures 637 -> 632 (the 5 owned cells),
gen-m9 5147 -> 5146, 0 regressions; both probes pass in full.
