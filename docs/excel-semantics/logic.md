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
(functions/logical/switch.rs) over an array or range expression, both through `lift_values` (functions/lift/driver.rs)
(T-0276). Tests: test/test_ifs_switch_lift.rs.

## XOR argument modes
Rule: XOR takes its arguments like AND (catalog mode `aggregate`). Text and blank cells in a range are skipped:
`XOR(A1:D1)` over 1, "x", TRUE, blank is FALSE. So is text inside an array: `XOR({1,"x",TRUE})` is FALSE. Direct text is
coerced: `XOR("TRUE")` is TRUE. A range with no logical value is `#VALUE!` (`XOR` of one text cell), while one number
cell (5) is TRUE. `=XOR()` makes Excel for the web refuse the workbook (`FileCorruptTryRepair`, scratch fill, not kept).
Evidence: probes/info-logic-argument-modes A1:A5, info-logic-argument-modes-2 A10:A11 (oracle: excel-graph, 2026-10-09, T-1088).
Status: VERIFIED (excel-graph); the engine matches.
Tasks: T-1088

## SWITCH: a blank cell case value does not match FALSE (T-2166)
Observation: `SWITCH(FALSE,1.005E-15,<#VALUE!>,A11,FALSE,"a?b",<#VALUE!>)` with A11 blank is `#N/A` (no match, no default):
FALSE does not match a blank cell given as a case value, unlike the omitted case argument below (`SWITCH(0,,"a","b")` is "a").
Ours matches it and returns FALSE (so `ISEVEN` of it is `#VALUE!` instead of `#N/A`). Probed (excel-graph, 2026-10-21, T-2200, probes/t2200-blank-and-padded-m5-args, Calc!A28:A46): a blank case cell matches only
a 0 or blank expression: `SWITCH(0,blank,..)` and `SWITCH(blank,blank,..)` hit it; FALSE, "" and TRUE do not (`#N/A`, no
default). A blank expression cell against a case matches 0 and a blank case cell, and not FALSE, "" or TRUE.
Evidence: gen-m5/d0419fa09d6a756a Calc!A63 (excel-graph, 2026-10-20); probes/t2200-blank-and-padded-m5-args.
Status: VERIFIED (excel-graph); implemented (T-2166: `switch_matches` reads a blank cell as 0 on either side; an
omitted case argument keeps the empty-value rule). Blank elements of a range expression or case
(`SWITCH(A1:A2,FALSE,..)`) take the same rule on the lifted path: a blank element matches 0 and a blank
cell, not FALSE or "" (probes/t2280-ifs-one-cell-range-condition Calc!A61:A67).

## IFERROR / IFNA of a blank cell is an empty value, not the reference
Rule: when IFERROR's or IFNA's value argument is a blank cell, the result is the blank's value, not a reference to it.
OR and AND read that value as FALSE (`OR(IFERROR(D9,A9))` with D9 blank is FALSE, whereas OR ignores a blank cell it is
given directly: `OR(B1)` is `#VALUE!`, `OR(B1,FALSE)` FALSE), `&` reads it as "" (`IFERROR(D5,A3)&" x "&"1"` is " x 1"), a
lookup reads it as 0 (`VLOOKUP(IFERROR(B8,D1),C8:D10,1,TRUE)` with B8 blank finds the largest value <= 0, -1;
`MATCH(IFERROR(B1,1),C1:C2,0)` over -1, 0 is 2), like an omitted lookup value (lookup.md, "XLOOKUP / XMATCH with an omitted
lookup_value is a blank cell"), and comparisons treat it as a blank: `IFERROR(B1,1)=0`, `=""`, `=FALSE`, `<1` and `<"a"`
are all TRUE and `SWITCH(IFERROR(B1,1),0,"zero",..)` is "zero". An omitted argument compares the same way:
`SWITCH(0,,"a","b")` and `SWITCH(,0,"a","b")` are "a", and `CHOOSE(1,)=0`, `=FALSE`, `=""`, `<1`, `IF(TRUE,)=""` and `IF(TRUE,)=0`
are TRUE. Unlike a blank cell given directly, COUNT and COUNTA
count it (`COUNT(IFERROR(B1,1))` = 1, COUNTA 1); so does an omitted argument (`COUNTA(,)` = 2, `COUNTA(1,)` = 2, `COUNTA(,,)` = 3, `COUNT(,)` = 2, `COUNT(1,)` = 2, `COUNTA(B1,)` = 1 with B1 blank, probes/counta-empty-value D1-D8, T-1294), and the count also holds through LET (`LET(x,IFERROR(B1,1),COUNTA(x))` = 1) and for two such arguments (2). The array form counts the empty element too (`COUNTA(IFERROR(B1:B2,1))` with B2 "x" is 2, D18; over a multi-cell range each blank is the number 0, so COUNT and AVERAGE count it: arrays.md "Array-value edges in the gen-m9 sweep", T-2579) whereas `COUNTA(TAKE(<blank>,1))` is 0 (probes/subtotal-reference-operands-c H13): TAKE of a range is a reference (references.md), so COUNTA skips its blank cells, while every element of an array value counts; the blank element of a lifted INDEX, an array, counts too (`COUNTA(INDEX(D1:D5,{1,2,3}))` with D2 blank is 3, probes/index-lifted-edge-cases H136), yet ISBLANK of it is TRUE, ISNUMBER and ISTEXT FALSE, TYPE 1, N 0,
and the cell itself shows 0 (as a consumer it reads as 0/FALSE/"": `NOT(IFERROR(B1,1))` TRUE, `IF(IFERROR(B1,1),..)` takes the false branch, `-IFERROR(B1,1)`, SUM, AVERAGE, MAX, PRODUCT and LEN of it are 0, `ISLOGICAL` FALSE, `LET(x,IFERROR(B1,1),ISBLANK(x))` TRUE; probes/counta-empty-value D9-D17, D25-D26). IFNA behaves the same (OR FALSE, `&` "", VLOOKUP 0).
Evidence: gen-m2 7469c3e920e1ade4 Calc!A351, ef5c55a81783b7d5 Calc!A291, dd99be1f2454159f Calc!A110 (excel-graph);
probes/iferror-blank-value D1:D23 and probes/omitted-arg-compare D1:D10 (excel-graph, 2026-10-09, T-0291).
A blank cell as the fallback IFERROR takes is the same empty value: STDEV.P counts `IFERROR(1/0,<blank>)` as 0
(probes/t2475-stat-subexpressions P!A158, excel-graph; statistics.md "Statistical argument-domain errors and extreme
values (T-2621)").
Ours: matches since T-0291 (`if_error_like` returns `CalcResult::EmptyArg`, for a blank fallback since T-2621; `lookup_key` reads it as 0 and
`compare_values` reads any `EmptyArg`, an omitted argument included, as a blank cell; before T-0291 the six omitted-argument
comparisons above were wrong). COUNTA reads its arguments in picked-reference context (TAKE, DROP, INDEX, OFFSET of a
range stay references) and counts every element of an array value, the empty ones included (T-1395). `CELL("contents", ref)` of a blank
target is the same empty value since T-3053, and array arguments of VSTACK, HSTACK, TOCOL, TOROW, TAKE, CHOOSECOLS, FILTER
and MAP reject it with `#VALUE!` (`VSTACK(IFERROR(Data!C11,5),1)`; information.md "CELL("contents", ref) as a VSTACK argument").
Status: VERIFIED (excel-graph)
Tasks: T-0235, T-0291, T-1294, T-1395
See also lookup.md "VLOOKUP and HLOOKUP hand a blank result cell to their caller as 0": VLOOKUP, HLOOKUP and LOOKUP
give the same empty value for a blank result cell, and IFS and SWITCH turn it into 0 (`IFS(TRUE,IFERROR(B2,1))&""` is
"0") while IF, CHOOSE and LET keep it (probes/lookup-blank-result-to-caller O1-O12, T-2168).

## IFNA edges: only #N/A is caught, lifts over arrays (excel-graph)
Rule: IFNA replaces only `#N/A` (`#DIV/0!`, `#VALUE!` pass through); a non-error value of any type is returned as is (`IFNA(5,1)` 5, `IFNA("",1)` ""). An omitted fallback (`IFNA(#N/A,)`) is 0, and an error fallback is returned (`IFNA(#N/A,#DIV/0!)` is `#DIV/0!`). It lifts elementwise: `IFNA({1,#N/A,3},0)` spills 1, 0, 3, `SUM(IFNA({1,#N/A,3},10))` is 14, `IFNA(#N/A,{1,2})` spills 1, 2, and `IFNA(Data!A1:A3,0)` over a range holding `#N/A` is `#SPILL!` in this probe only because Calc!A13 blocks the spill (range lifting itself is shown by the array-constant cells). `IFNA(VLOOKUP(9,..,FALSE),"none")` is "none", `IFNA(MATCH("z",..,0),0)` is 0, `IFNA(<blank cell>,1)` is 0 (see the blank rule above).
Evidence: probes/ifna-isformula-edges Calc!A1:A13, C1:E3 (oracle: excel-graph, 2026-10-12, T-1279).
Status: VERIFIED (excel-graph); the engine matches.
Tasks: T-1279

## IFS: a text condition gives a 1x1 array of #VALUE!; a one-cell range condition is a scalar (T-2280)
Rule: an IFS condition that is text not reading as a boolean ends IFS in a 1x1 array holding `#VALUE!`, not the scalar
error IF gives. This holds whether the text is a literal, a cell, a one-cell range or a formula result:
`TYPE(IFS("a*",5))`, `TYPE(IFS(A1,5))`, `TYPE(IFS(A1&"",5))` and `TYPE(IFS(FALSE,1,"a*",5))` are 64, and
`ROWS(IFS(A1,5)*2)` is 1 with A1 = "a*", while `ROWS(IF(A1,5)*2)` is `#VALUE!` and `TYPE(IF(A1,5)*2)` 16. The cell
itself shows `#VALUE!` and ISERROR of it is TRUE. Text that reads as a boolean ("TRUE") picks its value (5). An error or
blank condition and a FALSE-only `#N/A` stay scalar errors (TYPE 16). A one-cell range condition (`A3:A3`) is read as
its cell's scalar value, as IF reads it (references.md, T-2079): `ROWS(IFS(A3:A3,5))` with A3 FALSE is `#N/A`.
SWITCH has no such case: `ROWS(SWITCH(A1,TRUE,5)*2)` is `#N/A`.
Evidence: probes/t2280-ifs-one-cell-range-condition (excel-graph, 2026-10-21), Calc!A1:A60, A68:A80;
gen-m6/870c5e38f51ce5d4 Calc!A360 (`ROWS((IFS(Data!A$7,Data!D$10)*Data!C10))` is 1).
Status: VERIFIED (excel-graph); implemented (T-2280, `ifs_pick`).
Tasks: T-2280

## IFS: a text condition with an array value spills its shape (T-2653)
Rule: when an IFS condition is text not reading as a boolean, the `#VALUE!` result is an array of the broadcast shape of
that pair's value and every *later* argument (conditions and values), at least 1x1; earlier scalar pairs do not count (an earlier array condition lifts the whole call: `IFS({TRUE,FALSE},{1;2},"x",5)` is 2x2).
`IFS("x",{1,2;3,4})` is 2x2, `IFS("x",A1:B3)` 3x2, `IFS("x",5,TRUE,{1,2})` 1x2, `IFS("x",5,{TRUE,FALSE,TRUE},1)` 1x3,
`IFS("x",5,"y",{1,2,3,4})` 1x4, `IFS("x",{1,2},{TRUE;FALSE},3)` 2x2, `IFS("x",5)` 1x1; `IFS(FALSE,{1,2},"x",5)` is 1x1.
All have TYPE 64. A TRUE condition before the text one picks its value as usual. So a 3x2 value spills, or is `#SPILL!`
when blocked (gen-m5/4260ce3226165deb Calc!A208).
Evidence: probes/t2653-ifs-text-condition-array-value, probes/t2653-ifs-text-condition-shape-sources (excel-graph,
2026-10-25); gen-m5/4260ce3226165deb Calc!A208.
Untested: a later scalar error argument (`IFS("x",5,TRUE,1/0)`) and mismatched later shapes (`IFS("x",{1,2},TRUE,{1,2,3})`); the engine gives `#VALUE!` in every element.
Status: VERIFIED (excel-graph); implemented (T-2653, `ifs_text_condition_error`).
Tasks: T-2653

See also: IF error arguments are recorded in criteria.md "Zero and negative SUMIFS criteria; IF error arguments; RIGHT numeric-text num_chars" (T-2980).
