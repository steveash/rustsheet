# Logic

## IFS and SWITCH lift over arrays
Rule: IFS and SWITCH are element-wise over array/range conditions (and the SWITCH expression), and a range or array *value* argument is returned as is, so it spills: `IFS(TRUE,A1:B2)` and `SWITCH(1,1,A1:B2)` spill the 2x2 range; `IFS({TRUE,FALSE},1,TRUE,2)` = 1, 2; `IFS(A1:B2>1,5)` over 1,2;3,4 = `#N/A`,5;5,5; `SWITCH({1,2},1,"a",2,"b")` = a, b; `SWITCH(A1:B2,1,"a","z")` = a, z; z, z.
Evidence: probes/ifs-switch-array-values (oracle: excel-graph, 2026-10-07)
Status: VERIFIED
Unprobed: array value arguments beside a lifted condition or expression (the engine broadcasts them
element by element, as the element-wise operators do), and a position past a shorter array (the
engine reads it as an empty cell, as the *IF(S) criteria lift does).
Implemented in: parser/static_analysis.rs (`Function::Ifs`, `Function::Switch` are `Unknown`);
`fn_ifs` (functions/logical/mod.rs) lifts from the first array or range condition and `fn_switch`
(functions/logical/switch.rs) over an array or range expression, both through `lift_with_cost`
(T-0276). Tests: test/test_ifs_switch_lift.rs.
