---
id: T-1835
title: "EXPAND, WRAPROWS and WRAPCOLS lift their scalar arguments over a range or an array"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: [T-1770, T-1833, T-1834]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
EXPAND's rows, columns and pad_with and WRAPROWS's and WRAPCOLS's wrap_count and pad_with lift over a multi-cell range and an array, one call per element whose value is the top-left of that call's result, as in Excel; the array argument stays whole.

## Context
Split from T-1271. Mechanical once T-1834 lands: add table entries (the array argument `Whole`, the scalar ones as T-1833's per-parameter list says) and kernels using the element-value and cost rules T-1834 built; do not write a new lift or bound. Put the entries in their own `// --- M7 (T-1835) ---` block in functions/lift/kinds/ (mod.rs and the per-family files), as the engineering and math blocks are, so T-1836 and T-1839 can land in parallel. Delete the three functions' `fallback_signature`/`fallback_shape` arms in expressions/parser/static_analysis.rs.

Code: `fn_expand` in functions/lookup_and_reference/expand.rs, `fn_wrapcols`/`fn_wraprows` in functions/lookup_and_reference/wrapcols_wraprows.rs. T-1786 (done) already bounds one call's output with `check_array_size`; the kernel must keep that check and the function's argument-error order.

Owned cells:
- probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
  - F7:H7 `EXPAND(A1:A2,{3;4})`: ROWS 2, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
  - F8:H8 `WRAPROWS(A1:A3,{1;2})`: ROWS 2, COLUMNS 1, INDEX 1 (ours `#N/IMPL!`).
  - G42 `=COLUMNS(WRAPCOLS(A1:A3,2,{0;1}))` 1 (ours 2: pad_with is not lifted). F42 and H42 pass today and must keep passing.
  - K5 `=ROWS(INDEX(EXPAND(A1:A2,{3;4}),1,1))` 1; K6 `=ROWS(INDEX(WRAPROWS(A1:A3,{1;2}),1,1))` 1; K7 `=COLUMNS(INDEX(WRAPROWS(A1:A3,{1;2}),2,1))` 1.
- gen-m7/6db767d720c5b55a Calc!A12 `=WRAPCOLS(Data!D3:D3,UNIQUE(Data!B3:E4,SWITCH(..)))` (wrap_count is UNIQUE's 2-D array): Excel `#SPILL!`, ours `#N/IMPL!` (T-1275, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/6db767d720c5b55a.case.json`).

T-1960 probed the rest (probes/m7-lift-take-drop-sortby, excel-graph, 2026-10-21; Sheet1 rows 20-23, columns F:S as that case's description says): EXPAND's columns lifts (`EXPAND(A1:A2,3,{1;2})` 2x1, elements 1x1, value 1), WRAPROWS's pad_with lifts (`WRAPROWS(A1:A3,2,{0;1})` 2x1, element 2 is 1, ours a 2-wide result) and WRAPCOLS's wrap_count over an array literal lifts (`WRAPCOLS(A1:A3,{1;2})` 2x1); EXPAND's pad_with does **not** lift: `EXPAND(A1:A2,3,1,{0;1})` is one `#VALUE!` (ROWS, COLUMNS and INDEX of it `#VALUE!`), so pad_with is `Whole` and an array there is `#VALUE!` (SPEC-0011 per-parameter list). A 1x1 array (`EXPAND(A1:A2,{3})`) is T-2298's. `KernelWork::Output { weight: 1 }`.

Owned cells (T-1960):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/m7-lift-take-drop-sortby Sheet1!F20 | `=ROWS(EXPAND(A1:A2,3,{1;2}))` | 2.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!F21 | `=ROWS(EXPAND(A1:A2,3,1,{0;1}))` | #VALUE! | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!F23 | `=ROWS(WRAPCOLS(A1:A3,{1;2}))` | 2.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!G20 | `=COLUMNS(EXPAND(A1:A2,3,{1;2}))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!G21 | `=COLUMNS(EXPAND(A1:A2,3,1,{0;1}))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!G22 | `=COLUMNS(WRAPROWS(A1:A3,2,{0;1}))` | 1.0 | 2.0 |
| probes/m7-lift-take-drop-sortby Sheet1!G23 | `=COLUMNS(WRAPCOLS(A1:A3,{1;2}))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!H20 | `=INDEX(INDEX(EXPAND(A1:A2,3,{1;2}),1,1),1,1)` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!H21 | `=INDEX(INDEX(EXPAND(A1:A2,3,1,{0;1}),1,1),1,1)` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!H23 | `=INDEX(INDEX(WRAPCOLS(A1:A3,{1;2}),1,1),1,1)` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!I20 | `=ROWS(INDEX(EXPAND(A1:A2,3,{1;2}),1,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!I23 | `=ROWS(INDEX(WRAPCOLS(A1:A3,{1;2}),1,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!J20 | `=COLUMNS(INDEX(EXPAND(A1:A2,3,{1;2}),1,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!J23 | `=COLUMNS(INDEX(WRAPCOLS(A1:A3,{1;2}),1,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!K20 | `=SUM(INDEX(EXPAND(A1:A2,3,{1;2}),1,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!K23 | `=SUM(INDEX(WRAPCOLS(A1:A3,{1;2}),1,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!L20 | `=INDEX(INDEX(EXPAND(A1:A2,3,{1;2}),2,1),1,1)` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!L22 | `=INDEX(INDEX(WRAPROWS(A1:A3,2,{0;1}),2,1),1,1)` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!L23 | `=INDEX(INDEX(WRAPCOLS(A1:A3,{1;2}),2,1),1,1)` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!M20 | `=ROWS(INDEX(EXPAND(A1:A2,3,{1;2}),2,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!M23 | `=ROWS(INDEX(WRAPCOLS(A1:A3,{1;2}),2,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!N20 | `=COLUMNS(INDEX(EXPAND(A1:A2,3,{1;2}),2,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!N23 | `=COLUMNS(INDEX(WRAPCOLS(A1:A3,{1;2}),2,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!O20 | `=SUM(INDEX(EXPAND(A1:A2,3,{1;2}),2,1))` | 1.0 | #N/IMPL! |
| probes/m7-lift-take-drop-sortby Sheet1!O22 | `=SUM(INDEX(WRAPROWS(A1:A3,2,{0;1}),2,1))` | 1.0 | 3.0 |
| probes/m7-lift-take-drop-sortby Sheet1!O23 | `=SUM(INDEX(WRAPCOLS(A1:A3,{1;2}),2,1))` | 1.0 | #N/IMPL! |

Visibly-off cells (gap triage 2026-10-21, decision 0083): 11 cells, 1 numeric past half a unit in the 15th digit and 10 with a wrong type, error or text, blamed on WRAPROWS 5, EXPAND 4, ROWS 4, COLUMNS 4, WRAPCOLS 2, UNIQUE 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m7/6db767d720c5b55a Calc!A12 | `=WRAPCOLS(Data!D3:D3,UNIQUE(Data!B3:E4,SWITCH((-0.5),"a*",Data!$E9,Data!A12,1.005,Data!B9,Data!A6,(-0.0001))))` | #SPILL! | #N/IMPL! |
| probes/m7-argument-modes Sheet1!F7 | `=ROWS(EXPAND(A1:A2,{3;4}))` | 2 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!F8 | `=ROWS(WRAPROWS(A1:A3,{1;2}))` | 2 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!G42 | `=COLUMNS(WRAPCOLS(A1:A3,2,{0;1}))` | 1 | 2 |
| probes/m7-argument-modes Sheet1!G7 | `=COLUMNS(EXPAND(A1:A2,{3;4}))` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!G8 | `=COLUMNS(WRAPROWS(A1:A3,{1;2}))` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!H7 | `=INDEX(EXPAND(A1:A2,{3;4}),1,1)` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!H8 | `=INDEX(WRAPROWS(A1:A3,{1;2}),1,1)` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!K5 | `=ROWS(INDEX(EXPAND(A1:A2,{3;4}),1,1))` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!K6 | `=ROWS(INDEX(WRAPROWS(A1:A3,{1;2}),1,1))` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!K7 | `=COLUMNS(INDEX(WRAPROWS(A1:A3,{1;2}),2,1))` | 1 | #N/IMPL! |

gen-m7 cells (T-2225 triage, 2026-10-21): WRAPROWS/WRAPCOLS calls whose pad_with or wrap_count is an array (TOROW, FILTER or UNIQUE result): Excel lifts (a `#NUM!`/`#VALUE!` grid, or `#SPILL!` for 6db767d720c5b55a A12), ours `#N/IMPL!` or one cell.

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 28 known-failure lines in eval/baseline/gen-m7.txt, 3 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/611ce3bbfab27cef.case.json A1176 `=WRAPROWS(Data!E11:E12,Data!$A$4,((Data!D9-Data!D5)-TOROW(Data!B6:E8,1,Data!E1)))`: B1176 Excel #NUM!, ours <empty>; cells (4): B1176:E1176
- gen-m7/6db767d720c5b55a.case.json A12 `=WRAPCOLS(Data!D3:D3,UNIQUE(Data!B3:E4,SWITCH((-0.5),"a*",Data!$E9,Data!A12,1.005,Data!B9,Data!A6,(-0.0001))))`: A12 Excel #SPILL!, ours #N/IMPL!; cells (1): A12
- gen-m7/90de8755a2e565e6.case.json A889 `=WRAPROWS(((Data!B5+Data!C7)-SIGN(Data!B4)),(-1),((Data!B4-(-0.5))*FILTER(Data!A1:D12,(Data!B1:B12>3.14159))))`: B889 Excel #VALUE!, ours <empty>; cells (23): B889:D889, A890:D890, A891:D891, A892:D892, A893:D893, A894:D894

## Acceptance criteria
- [ ] The 28 gen-m7 cells listed under Context (T-2225 triage) match Excel: their 28 known-failure lines leave eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known failures.
- [ ] Visible parity: the 11 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 10 lines in eval/baseline/probes.txt disappear (`make eval-baseline`); the 1 cell of the ungated gen-m7 suite passes in `make eval`, with no new known-failure lines in any baseline.
- [ ] probes/m7-argument-modes F7:H7, F8:H8, G42 and K5-K7 pass, and F42, H42 still pass.
- [ ] The probes/m7-lift-take-drop-sortby cells listed under Context pass (EXPAND's array pad_with is `#VALUE!`).
- [ ] gen-m7/6db767d720c5b55a A12 passes, with no new failing gen-m7 cell.
- [ ] Their lines leave eval/baseline/ (`make eval-baseline`), with no new known failures.
- [ ] The three functions have table entries citing those cells; their fallback arms are gone (`no_function_in_the_table_has_a_fallback_arm` passes).
- [ ] Unit tests: an unlifted call is unchanged (value and static shape); `=ROWS(EXPAND(1,SEQUENCE(1024),1024))` is `#ERROR!` (SPEC-0011 rule 3); an element error stays in its position.
- [ ] `make check` green; reviewers approve.

## Out of scope
- WRAPCOLS of a single value and wrap_count below 1 (T-1348).
- Any change to the lift driver or the element rules (T-1834); if they do not fit, hand the task up (work-loop 5b).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#N/IMPL!` and a wrong width where Excel lifts EXPAND, WRAPROWS and WRAPCOLS), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
