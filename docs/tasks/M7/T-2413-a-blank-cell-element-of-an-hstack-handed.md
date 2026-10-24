---
id: T-2413
title: "A blank-cell element of an HSTACK handed to a REDUCE LAMBDA counts as 0 in AVERAGE (gen-m7)"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
An element of an array value that comes from a blank cell (here through HSTACK), handed to a REDUCE LAMBDA, is counted as 0 by
AVERAGE the way Excel counts it, so `REDUCE(<3x1 text array>,HSTACK(Data!Z1:Z2,{1;2}),_xleta.AVERAGE)` is 1.125.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 1 cell.
`REDUCE(REPLACE(INDIRECT("Data!A2:A4",TRUE),3,2,Data!B10),HSTACK(Data!Z1:Z2,{1;2}),_xleta.AVERAGE)` is 1.125 (f19581ac44da1653 A2088).
The initial value is a 3x1 array of text, which AVERAGE skips inside an array; the elements are Z1 (blank), 1, Z2 (blank), 2 in row
order. 1.125 is AVERAGE(AVERAGE(AVERAGE(AVERAGE(texts,0),1),0),2) = (((0+1)/2+0)/2+2)/2, so the first blank element counts as a 0 argument.
Ours `#DIV/0!`: the blank element of the HSTACK array reaches AVERAGE as a blank that it skips (`COUNT(INDEX(HSTACK(Z1:Z2,{1;2}),1,1))` is
0 and `ISBLANK` of the element TRUE in ours), so the first step averages nothing. With a literal `{0,1;0,2}` ours gives 1.125, and AVERAGE
already counts the empty value of `IFERROR(Z1,1)` (logic.md "IFERROR / IFNA of a blank cell is an empty value"). Unknown: whether
Excel's HSTACK turns a blank cell into 0 (its spill shows 0) or keeps an empty value that a function argument counts; probe
`ISBLANK(INDEX(HSTACK(Z1:Z2,{1;2}),1,1))`, `COUNT(..)`, `INDEX(..)&"|"`, the same for VSTACK, and `MAP(HSTACK(Z1:Z2,{1;2}),_xleta.COUNT)`
before choosing the fix (excel-probe skill). Code: `fn_hstack`/`fn_vstack` (blank padding/copying) or the LAMBDA argument binding in
functions/logical/map_reduce.rs. Quirk entry: docs/excel-semantics/arrays.md "SCAN and REDUCE keep errors in the LAMBDA" (sentence added by
this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/f19581ac44da1653.case.json A2088 `=REDUCE(REPLACE(INDIRECT("Data!A2:A4",TRUE),3,2,Data!B10),HSTACK(Data!Z1:Z2,{1;2}),_xleta.AVERAGE)`: A2088 Excel 1.125, ours #DIV/0!; cells (1): A2088

## Acceptance criteria
- [x] A probe case with the cells above, filled from excel-graph; arrays.md records which reading Excel has.
- [x] Visible parity: this 1 known-failure line disappears from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline; every probe cell passes or has its own task (seven: T-2461).
- [x] `make check` green; reviewers approve.

## Out of scope
- A lone blank cell reference given to TOROW or UNIQUE (T-2409).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probe probes/t2413-blank-in-stacked-arrays (45 cells, excel-graph, 2026-10-23): a blank cell taken into an array value by HSTACK,
VSTACK or an EXPAND pad is the empty value IFERROR gives a blank cell (ISBLANK TRUE, `&` "", but COUNT, COUNTA and AVERAGE count
it as 0); a range's blank cell stays a blank cell. `lambda_cells` now binds a blank array element as `CalcResult::EmptyArg`, so
MAP, REDUCE and SCAN see the empty value (unit test `blank_element_of_an_array_value_is_the_empty_value_in_a_lambda`).
Making every array-element read give it (`array_node_to_calc_result`) regressed TEXTJOIN, IFS and TOCOL cells, so INDEX's read,
VSTACK's blank copy and EXPAND's blank pad (7 probe cells) are T-2461. arrays.md records the rule.
Eval delta: gen-m7 known failures -1 (f19581ac44da1653 A2088); probes +1 case with 7 new-coverage lines (T-2461); 0 regressions.
