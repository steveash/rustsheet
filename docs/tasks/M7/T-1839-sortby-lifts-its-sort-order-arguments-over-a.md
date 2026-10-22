---
id: T-1839
title: "SORTBY lifts its sort_order arguments over a range or an array (alternating by_array/sort_order kinds in the lifting table)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-1770, T-1833, T-1834, T-1960, T-2298, T-2300]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SORTBY's sort_order arguments lift over a multi-cell range and an array, one call per element whose value follows the element rule T-1960 established (Context: uniform entries give the top-left of each call, otherwise later elements are empty or `#VALUE!`; probes/sort-error-args H97-H99), while its array and by_array arguments stay whole, as in Excel; the lifting table can describe SORTBY's alternating (by_array, sort_order) pairs (SPEC-0011 rule 4).

## Context
Split from T-1271. `ArgKinds` has a single `repeat` kind, and SORTBY's arguments after the first alternate `Whole` by_array and `Lift` sort_order; SPEC-0011 "Array-valued functions" rule 4 (T-1833) makes `repeat` a cycle of kinds (`&'static [ArgKind]`, whole cycles only in `accepts`); SORTBY is `params: [Whole, Whole, Lift]`, `repeat: [Whole, Lift]`. This task builds it (functions/lift/kinds/ (mod.rs and the per-family files), and `lifted_static_result`/`get_function_args_signature` where they read it) with SORTBY as its consumer. Each element sorts the whole array, so charge it the array's and by_arrays' cells per element (`lift_call_costed`) (SPEC-0011 rule 3). Put the entry in its own `// --- M7 (T-1839) ---` block so T-1835 and T-1836 can land in parallel. Delete SORTBY's `fallback_signature`/`fallback_shape` arms in expressions/parser/static_analysis.rs.

Code: `fn_sortby` in functions/spill_functions.rs.

Owned cells, probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
- F26:H26 `SORTBY(A1:A3,B1:B3,{1;-1})`: ROWS 2, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
- probes/sort-error-args H97-H99 `SORTBY(A1:A3,C1:C3,{1;-1;1})` is {1;0;0} (moved here from T-1902): SPEC-0011's top-left rule gives {1;2;1}, so the element rule comes from T-1960's probe.
F56:H56 (`SORTBY(A1:A3,B1:B2)` is `#VALUE!`) passes today and must keep passing.

The element rule (T-1960, probes/m7-lift-take-drop-sortby sheet Sort, excel-graph, 2026-10-21; A1:A3 = 3,1,2, B1:B3 = 30,10,20, C1:C3 = 2,1,3, D1:D3 = c,a,b, E1:E3 = 1,-1,1, F1:F3 = 1,2,3; arrays.md "UNIQUE's and SORTBY's lifted elements"): the result has one element per entry. When every entry is equal each element is the top-left of that call (`{1;1;1}` 1,1,1, `{-1;-1;-1}` 2,2,2, `{1;1}+0` 1,1). Otherwise element 1 is the top-left of the call with the first entry and every later element is **empty** (spills as 0; ISBLANK TRUE, SUM 0, ROWS, COLUMNS and INDEX of it `#VALUE!`), even where its entry equals the first (`{1;-1;1}` gives 1, empty, empty: H97-H99's {1;0;0}; `{1;1;-1}` 1, empty, empty; a range E1:E3 the same), unless an entry is not 1 or -1 (`{1;2}`, `{1;"x"}`: that element is `#VALUE!`) or the call has a later by_array/sort_order pair (`SORTBY(A1:A3,C1:C3,{1;-1},A1:A3,1)` and `SORTBY(A1:A3,C1:C3,1,A1:A3,{1;-1})`: element 2 is `#VALUE!`, SUM `#VALUE!` too). A later sort_order lifts the same way when uniform (`1,A1:A3,{1;1}` and `{-1;-1}`: two elements, 1). A row `{1,-1}` is 1x2; `{1}` and `{-1}` are 1x1 (T-2298's 1x1 rule). The empty-later-elements rule is UNIQUE's too: reuse T-2300's driver option, do not copy it. The case's Sort sheet (452 failing cells today) is owned here.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 4 cells, 0 numeric past half a unit in the 15th digit and 4 with a wrong type, error or text, blamed on SORTBY 4, ROWS 1, COLUMNS 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/m7-argument-modes Sheet1!F26 | `=ROWS(SORTBY(A1:A3,B1:B3,{1;-1}))` | 2 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!G26 | `=COLUMNS(SORTBY(A1:A3,B1:B3,{1;-1}))` | 1 | #N/IMPL! |
| probes/m7-argument-modes Sheet1!H26 | `=INDEX(SORTBY(A1:A3,B1:B3,{1;-1}),1,1)` | 1 | #N/IMPL! |
| probes/sort-error-args Sheet1!H97 | `=SORTBY(A1:A3,C1:C3,{1;-1;1})` | 1 | #N/IMPL! |

gen-m7 cells (T-2225 triage, 2026-10-21): SORTBY calls whose sort_order (or a later by_array/sort_order pair) is an array or range, so the call lifts: Excel spills one element per entry (mostly `#VALUE!`, `#N/A` or `#REF!` grids), ours is `#N/IMPL!` or a single cell.

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 80 known-failure lines in eval/baseline/gen-m7.txt, 8 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/07d27aa0add0247f.case.json A1596 `=SORTBY(Data!B1:D12,Data!D1:D12,1,ERROR.TYPE(Data!D7),0,NOT(FALSE),FILTER(Data!B1:D12,(Data!A1:A12<>0),Data!A3:D5))`: B1596 Excel #N/A, ours <empty>; cells (35): B1596:C1596, A1597:C1597, A1598:C1598, A1599:C1599, A1600:C1600, A1601:C1601, A1602:C1602, A1603:C1603, A1604:C1604, A1605:C1605, A1606:C1606, A1607:C1607
- gen-m7/18b80d3fd292d63a.case.json A975 `=SORTBY(Data!A1:D12,(MIN(Data!B6:B10)+INDEX(Data!D10:E11,2)),XLOOKUP((Data!C1/42),Data!B10:B11,Data!B7:D9,EXPAND(Data!A8,Data!D8,5,7),UNIQUE(Data!A11:C11,Data!A$1,TRUE),-2))`: B975 Excel #REF!, ours <empty>; cells (2): B975:C975
- gen-m7/2c5ea078f25443a2.case.json A1250 `=SORTBY(Data!A1:B12,HOUR((Data!E5*3.14159)),-1,SUMIF(Data!B3:D8,"",Data!B6:D10),(ROUNDDOWN((-0.0001),Data!B2)-SORT(Data!D1:E1)),2.5,COUNT(Data!D5:D6,INDEX(Data!B7:C10,Data!C11,3)))`: B1250 Excel #VALUE!, ours <empty>; cells (1): B1250
- gen-m7/36809e9faa59553f.case.json A13 `=SORTBY(Data!B1:E12,Data!$B3,HSTACK(Data!B7:E7,TRUNC(0.1,8),Data!A7:C7),COUNTIF(Data!A3:A12,""),0,((Data!A12-Data!D2)>("a"<=Data!C7)),1)`: A13 Excel #VALUE!, ours #N/IMPL!; cells (5): A13:E13
- gen-m7/611ce3bbfab27cef.case.json A1337 `=SORTBY(Data!C1:E12,Data!A1:A12,VLOOKUP(NOT(FALSE),Data!C1:D6,Data!$B12,OFFSET(Data!A7:A10,6,0)),Data!D1:D12,((Data!$D$3/(-0.0001))-Data!E1))`: A1337 Excel #N/A, ours #N/IMPL!; cells (4): A1337, A1338, A1339, A1340
- gen-m7/8044dce50616cfe9.case.json A198 `=SORTBY(Data!A1:D12,MAX(42,Data!C8:C10,Data!D11:D12),Data!B11,((Data!$D10-Data!D3)=(Data!E12>Data!B5)),1,Data!A1:A12,SCAN(REDUCE(Data!E7,Data!C3:E10,LAMBDA(y,z,Data!$C$12)),Data!A7:D10,_xleta.MIN))`: A198 Excel #VALUE!, ours #N/IMPL!; cells (16): A198:D198, A199:D199, A200:D200, A201:D201
- gen-m7/8044dce50616cfe9.case.json A1050 `=SORTBY(Data!D1:E12,Data!D1:D12,0,(((-1)*Data!$E6)+PI()),(EXPAND(Data!A2:A2,3,5," x ")/COUNTIF(Data!A3:C4,"")))`: A1050 Excel #DIV/0!, ours #N/IMPL!; cells (15): A1050:E1050, A1051:E1051, A1052:E1052
- gen-m7/b24ce74f08d3f91a.case.json A581 `=SORTBY(Data!E1:E12,Data!B1:B12,42,COUNT(Data!D6:D11),WEEKNUM(40212),"ü",IF(ISNUMBER(Data!C8),INDEX(Data!B6:D12,1,Data!D5),FIND(Data!B2,Data!B11)))`: B581 Excel #VALUE!, ours <empty>; cells (2): B581:C581

## Acceptance criteria
- [ ] The 80 gen-m7 cells listed under Context (T-2225 triage) match Excel: their 80 known-failure lines leave eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known failures.
- [ ] Visible parity: the 4 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 4 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Sheet Sort of probes/m7-lift-take-drop-sortby passes.
- [ ] probes/m7-argument-modes F26:H26 and probes/sort-error-args H97-H99 pass and F56:H56 still pass; their lines leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] SORTBY has a table entry citing those cells; its fallback arms are gone (`no_function_in_the_table_has_a_fallback_arm` passes); a unit test checks the kinds of arguments 1-6 and that the accepted argument counts are those `fn_sortby` accepts today (2, 3, then odd counts from 5; an even count above 3 is the count error).
- [ ] Unit tests: an unlifted SORTBY is unchanged (value and static shape); a lifted sort_order2 in a two-key call lifts; a by_array given as an array literal is not lifted; a lifted call over a large array is `#ERROR!` past the visit cap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT (T-1837); SORT's and SORTBY's argument-error order (T-1351).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#N/IMPL!` where Excel lifts SORTBY), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
