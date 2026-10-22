# 0100 — LAMBDA calls nest 300 deep: smaller frames, the same budget

- **Status:** accepted (2026-10-22, T-0444); amends the consequences of decision 0080.
- **Context:** Decision 0080 lets nested LAMBDA calls use four times the evaluation's stack budget
  (1 MB by default) and listed the 300-level probe cells (probes/lambda-call-depth H3, H6, I3, I6)
  as deviations, since only about 200 levels fit in a release build. Excel evaluates 300. Measured
  with gdb on the release test binary, a level of `LET(f,LAMBDA(g,n,IF(n=0,0,g(g,n-1)+1)),f(f,n))`
  took 5,424 bytes, 3,792 of them three `evaluate_node_in_context` frames of 1,264 bytes each: the
  locals of rarely used node arms (defined names, the spill operator, `@`, unions, 3-D spans),
  of the operator chain and of the per-reference-element lift were inlined into it.
- **Decision:** Keep the budget (4 × `set_max_evaluation_stack`, 1 MB) and make the frames smaller
  instead: `evaluate_node_in_context` answers only the nodes nested calls go through and hands the
  rest to an out-of-line `evaluate_cold_node`; the operator chain, a chain link's operator, a call
  of a name and `call_per_reference_element` are out of line. A level now takes about 2,560 bytes:
  408 self-calls and 491 REDUCE closures nest in a release build (116 and 147 in a debug build),
  so the 300-level probe cells pass and their lines leave `eval/suites/probes/deviations`.
- **Consequences:** The default still needs about 2 MB of host stack, as decision 0080 says, and a
  call 20,000 deep is still `#NUM!`. The budget stays a directive-5 DEVIATION past about 400 levels,
  where Excel's own behavior is unknown (docs/excel-semantics/arrays.md "How deep LAMBDA calls nest").
  A change that inlines a large callee back into `evaluate_node_in_context` makes the probes
  regress, which the eval gate catches.
