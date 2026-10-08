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
