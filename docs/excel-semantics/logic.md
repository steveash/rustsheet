# Logic

## IFS and SWITCH lift over arrays
Rule: IFS and SWITCH are element-wise over array/range conditions (and the SWITCH expression), and a range or array *value* argument is returned as is, so it spills: `IFS(TRUE,A1:B2)` and `SWITCH(1,1,A1:B2)` spill the 2x2 range; `IFS({TRUE,FALSE},1,TRUE,2)` = 1, 2; `IFS(A1:B2>1,5)` over 1,2;3,4 = `#N/A`,5;5,5; `SWITCH({1,2},1,"a",2,"b")` = a, b; `SWITCH(A1:B2,1,"a","z")` = a, z; z, z.
Evidence: probes/ifs-switch-array-values
Status: PARTIAL (value-range spill implemented in T-0272; array conditions/expression are a scalar path, T-0276)
Implemented in: parser/static_analysis.rs (`Function::Ifs`, `Function::Switch` are `Unknown`)
