---
id: T-2298
title: "A 1x1 array lifts in an array-valued function's lifted parameter (delete T-1834's 1x1 rule)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1960]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A 1x1 array in a lifted parameter of an array-valued function (SEQUENCE, MAKEARRAY, EXPAND, TAKE, DROP, TOCOL, UNIQUE, SORTBY, ...) lifts to one element, as Excel does, while a single-cell reference or a value derived from one still runs the scalar call and keeps its whole array.

## Context
From T-1960. T-1834 added a rule the spec did not name: an `Array` entry whose lifted arguments are all 1x1 runs one unlifted call and keeps its whole array (`lift_grids` in functions/lift/driver.rs, the `options.result == CallResult::Array && (rows, columns) == (1, 1)` branch; SPEC-0011 "As built (T-1834)"). Excel lifts a 1x1 array: `SEQUENCE({3})` is one element, 1 (ROWS 1, it spills one cell), `SEQUENCE({3},2)` is 1x1, `SEQUENCE(A3+{0})`, `EXPAND(A1:A2,{3})`, `TOCOL(A1:B3,{0})` and `UNIQUE(D1:D3,{FALSE})` are 1x1, and `TAKE(A1:B3,{2})` is one reference element A1:B2 (ROWS and COLUMNS of `INDEX(..,1,1)` 2, SUM 33; spilled directly `#VALUE!`, as a lifted OFFSET's element). A single cell is not an array there: `SEQUENCE(A3*1)`, `SEQUENCE(INDIRECT("A3")*1)`, `SEQUENCE(A3:A3)`, `SEQUENCE(IF(TRUE,A3))` and `SEQUENCE(INDEX({3},1,1))` with A3 = 3 spill three rows (rows 38-42 pass today and must keep passing). So the branch must tell a 1x1 array value apart from a one-cell reference (and arithmetic over one), which the driver collapses into `LiftedArg::Grid` today; decide where (`LiftedArg::from` or the caller) without a second lift path. SORTBY's `{1}` (Sort!F43) is T-1839's, which depends on this task.

Owned cells, probes/m7-lift-take-drop-sortby (T-1960, excel-graph, 2026-10-21; Sheet1 A1:B3 = 1,10;2,20;3,30, D1:D3 = x,y,x; column E holds each row's formula as text, F:S its ROWS, COLUMNS and per element INDEX(INDEX(..,i,1),1,1), ROWS, COLUMNS and SUM of INDEX(..,i,1); arrays.md "Dynamic-array and LAMBDA function argument modes"):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/m7-lift-take-drop-sortby Sheet1!F12 | `=ROWS(SEQUENCE({3}))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F13 | `=ROWS(TAKE(A1:B3,{2}))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F14 | `=ROWS(SEQUENCE({3},2))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F15 | `=ROWS(EXPAND(A1:A2,{3}))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F43 | `=ROWS(SEQUENCE(A3+{0}))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F44 | `=ROWS(TOCOL(A1:B3,{0}))` | 1.0 | 6.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F45 | `=ROWS(UNIQUE(D1:D3,{FALSE}))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!G13 | `=COLUMNS(TAKE(A1:B3,{2}))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!G14 | `=COLUMNS(SEQUENCE({3},2))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!I13 | `=ROWS(INDEX(TAKE(A1:B3,{2}),1,1))` | 2.0 | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!J13 | `=COLUMNS(INDEX(TAKE(A1:B3,{2}),1,1))` | 2.0 | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!K13 | `=SUM(INDEX(TAKE(A1:B3,{2}),1,1))` | 33.0 | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!U19 | `=ROWS(SEQUENCE({3},2))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!U20 | `=COLUMNS(TAKE(A1:B3,{2}))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!U21 | `=ROWS(TAKE(A1:B3,{2}))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!U6 | `=ROWS(SEQUENCE({3}))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!W65 | `=TAKE(A1:B3,{2})` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!W61 | `(spilled cell)` | <empty> | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!W62 | `(spilled cell)` | <empty> | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!X65 | `(spilled cell)` | <empty> | 10.0 |
| probes/m7-lift-take-drop-sortby Sheet1!W66 | `(spilled cell)` | <empty> | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!X66 | `(spilled cell)` | <empty> | 20.0 |

gen-m7 cells (T-2225 triage, 2026-10-21): A 1x1 array (BYROW of one cell, MINVERSE of a scalar, a one-cell SCAN, SORT of one cell) in a lifted parameter of SEQUENCE, WRAPROWS, EXPAND and TOCOL: Excel lifts it to one element, ours keeps the whole array (or `#SPILL!`).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 6 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/18b80d3fd292d63a.case.json A1753 `=EXPAND(((0.5/Data!D$6)+2),SCAN(IF(FALSE,"TRUE",Data!D9),Data!D10:D10,_xleta.MAX))`: A1754 Excel <empty>, ours #N/A; cells (1): A1754
- gen-m7/7307132bb8820f17.case.json A422 `=SEQUENCE(4,2,((100+Data!A$2)*BYROW(Data!D7,LAMBDA(v,v))),(AVERAGE(Data!A4,123456789.123,0.2)*MAX(Data!B5:B6)))`: A422 Excel 100.0, ours #SPILL!; cells (1): A422
- gen-m7/8044dce50616cfe9.case.json A1780 `=WRAPROWS(Data!D10:D11,1,MINVERSE(SUMPRODUCT(Data!A10:B12)))`: A1781 Excel <empty>, ours 0.3; cells (1): A1781
- gen-m7/c36ff5f42a1a4d47.case.json A804 `=MAP(TOCOL(Data!B7:E12,1,SORT(Data!$C$4)),Data!A11:C11,LAMBDA(x,v,DAY(1e-15)))`: A804 Excel 0.0, ours #SPILL!; cells (3): A804:C804

## Acceptance criteria
- [ ] The 6 gen-m7 cells listed under Context (T-2225 triage) match Excel: their 6 known-failure lines leave eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known failures.
- [ ] The cells above pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures; Sheet1 rows 38-42 still pass.
- [ ] SPEC-0011 "As built (T-1834)" and the driver comment say the 1x1 rule is gone and cite the probe.
- [ ] Unit tests: `SEQUENCE({3})` is a 1x1 lifted result; `SEQUENCE(A3:A3)` and `SEQUENCE(A3*1)` keep spilling 3 rows.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY (T-1839).

## Outcome
The driver's 1x1 rule is gone from `lift_grids` and `lift_references`: a 1x1 grid lifts like any other. A single cell never reaches the driver as a grid: a one-cell range was already its value in the read phase, and an arithmetic operator now reads a one-cell range operand as its cell's value (`arithmetic_operand`, arithmetic.rs), so `SEQUENCE(INDIRECT("A3")*1)` keeps spilling (row 39, the one regression of deleting the rule alone). TAKE's and DROP's counts lift a 1x1 array (`count_lifts`); OFFSET, INDIRECT and TRIMRANGE keep `lifts` (unprobed). Static shape of a 1x1 lift stays `Unknown`. SPEC-0011 "As built (T-1834)" and arrays.md updated.

Eval delta: probes known failures 2446 -> 2425 (21 lines fixed), gen-m7 1068 -> 1052 (16 lines fixed), 0 regressions; rows 38-42 still pass. Not fixed here: Sheet1!F15 (EXPAND), gen-m7 A1754 (EXPAND) and A1781 (WRAPROWS), because EXPAND and WRAPROWS are not lifted on main yet; they pass through this driver change once T-1835 lifts them, and are handed to T-1835's Context.
