# Logic

## IFS and SWITCH lift over arrays
Rule: IFS and SWITCH are element-wise over array/range conditions (and the SWITCH expression), and a range or array *value* argument is returned as is, so it spills: `IFS(TRUE,A1:B2)` and `SWITCH(1,1,A1:B2)` spill the 2x2 range; `IFS({TRUE,FALSE},1,TRUE,2)` = 1, 2; `IFS(A1:B2>1,5)` over 1,2;3,4 = `#N/A`,5;5,5; `SWITCH({1,2},1,"a",2,"b")` = a, b; `SWITCH(A1:B2,1,"a","z")` = a, z; z, z.
Evidence: probes/ifs-switch-array-values (oracle: excel-graph, 2026-10-07), probes/ifs-switch-lift-edges
(excel-graph, 2026-10-08, T-0276)
Status: VERIFIED
Value arguments beside a lifted condition or expression broadcast element by element, arrays and
ranges alike: `IFS({TRUE,FALSE},{10,20},TRUE,{30,40})` and `SWITCH({1,2},1,{10,20},{30,40})` are
10, 40, and `IFS(A1:B2>1,A1:B2)` is `#N/A`,2;3,4. Scalar conditions before the first array one still
short-circuit: `IFS(TRUE,1,{TRUE,FALSE},2)` is a single 1 and `IFS(1/0,1,{TRUE},2)` is `#DIV/0!`.
A single-cell range condition or expression gives the same results as a scalar one (`IFS(C1:C1,5)`,
`ROWS(IFS(C1:C1,{5;6}))` = 2, `SWITCH(D1:D1,1,"a","z")`). Ragged shapes stretch a size-1 dimension and
are `#N/A` past a shorter array: `IFS({TRUE,FALSE,TRUE},{1,2},TRUE,0)` is 1, 0, `#N/A`;
`SWITCH({1,2,3},1,{"a","b"},"z")` is a, z, z. An array case list broadcasts against the expression:
`SWITCH({1;2},{1,2},"m","n")` is m, n; n, m.
Implemented in: parser/static_analysis.rs (`Function::Ifs`, `Function::Switch` are `Unknown`);
`fn_ifs` (functions/logical/mod.rs) lifts from the first array or range condition and `fn_switch`
(functions/logical/switch.rs) over an array or range expression, both through `lift_with_cost`
(T-0276). Tests: test/test_ifs_switch_lift.rs.

## XOR argument modes
Rule: XOR takes its arguments like AND (catalog mode `aggregate`). Text and blank cells in a range are skipped:
`XOR(A1:D1)` over 1, "x", TRUE, blank is FALSE. So is text inside an array: `XOR({1,"x",TRUE})` is FALSE. Direct text is
coerced: `XOR("TRUE")` is TRUE. A range with no logical value is `#VALUE!` (`XOR` of one text cell), while one number
cell (5) is TRUE. `=XOR()` makes Excel for the web refuse the workbook (`FileCorruptTryRepair`, scratch fill, not kept).
Evidence: probes/info-logic-argument-modes A1:A5, info-logic-argument-modes-2 A10:A11 (oracle: excel-graph, 2026-10-09, T-1088).
Status: VERIFIED (excel-graph); the engine matches.
Tasks: T-1088

## IFERROR / IFNA of a blank cell is an empty value, not the reference
Rule: when IFERROR's or IFNA's value argument is a blank cell, the result is the blank's value, not a reference to it.
OR and AND read that value as FALSE (`OR(IFERROR(D9,A9))` with D9 blank is FALSE, whereas OR ignores a blank cell it is
given directly and is then `#VALUE!` with nothing else), `&` reads it as "" (`IFERROR(D5,A3)&" x "&"1"` is " x 1"), a
lookup reads it as 0 (`VLOOKUP(IFERROR(B8,D1),C8:D10,1,TRUE)` with B8 blank finds the largest value <= 0, -1;
`MATCH(IFERROR(B1,1),{-1;0},0)` is 2), like an omitted lookup value (lookup.md, "XLOOKUP / XMATCH with an omitted
lookup_value is a blank cell"), and comparisons treat it as a blank: `IFERROR(B1,1)=0`, `=""`, `=FALSE`, `<1` and `<"a"`
are all TRUE and `SWITCH(IFERROR(B1,1),0,"zero",..)` is "zero". Unlike a blank cell given directly, COUNT and COUNTA
count it (`COUNT(IFERROR(B1,1))` = 1, COUNTA 1), yet ISBLANK of it is TRUE, ISNUMBER and ISTEXT FALSE, TYPE 1, N 0,
and the cell itself shows 0. IFNA behaves the same (OR FALSE, `&` "", VLOOKUP 0).
Evidence: gen-m2 7469c3e920e1ade4 Calc!A351, ef5c55a81783b7d5 Calc!A291, dd99be1f2454159f Calc!A110 (excel-graph);
probes/iferror-blank-value D1:D23 (excel-graph, 2026-10-09, T-0291).
Ours: matches since T-0291 (`if_error_like` returns `CalcResult::EmptyArg`; `lookup_key` reads it as 0 and
`compare_values` as a blank cell) except COUNTA, which is 0 (T-1294).
Status: VERIFIED (excel-graph)
Tasks: T-0235, T-0291, T-1294
