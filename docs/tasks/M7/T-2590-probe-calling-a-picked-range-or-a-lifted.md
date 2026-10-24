---
id: T-2590
title: "Probe calling a picked range or a lifted reference result: IFS(TRUE,A1:A3)(2), TAKE(A1:A3,{1;2})(2)"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value for a call of a range that IFS or SWITCH picks, of a lifted TAKE or DROP, and of a
TAKE or XLOOKUP over an array is evidenced by excel-graph probe cases, and ours matches or the gap is a task.

## Context
T-2499 made the callee of a written call (`callee(args)`) read as SUBTOTAL reads it
(`Model::evaluate_callee`, functions/logical/lambda.rs, via `evaluate_picked_reference_arg`). That also
changes shapes no probe covers (Excel-fidelity review of T-2499): `IFS(TRUE,A1:A3)(2)` and
`SWITCH(1,1,A1:A3)(2)` are now `#REF!` (a picked range) where they were read as values, and a lifted
`TAKE(A1:A3,{1;2})(2)` hands an array of references to `call_variable`. Array sources stay values:
`TAKE({1,2,3},1)(2)` and `XLOOKUP(1,{1,2},{1,2})(2)` (expected `#NAME?`, a called number).
Existing evidence: probes/chained-calls-builtin-* (excel-graph, T-2405, T-2499);
docs/excel-semantics/references.md "Calling a call's result".

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with oracle values for `=IFS(TRUE,A1:A3)(2)`, `=SWITCH(1,1,A1:A3)(2)`,
      `=TAKE(A1:A3,{1;2})(2)`, `=DROP(A1:A3,{1;2})(2)`, `=TAKE({1,2,3},1)(2)` and `=XLOOKUP(1,{1,2},{1,2})(2)`.
- [ ] references.md "Calling a call's result" records the findings; any mismatch gets an engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (a follow-up task if a mismatch shows).
