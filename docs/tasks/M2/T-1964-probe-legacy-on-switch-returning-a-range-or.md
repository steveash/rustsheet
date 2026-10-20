---
id: T-1964
title: "Probe legacy @ on SWITCH returning a range or nested under scalar functions"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's FORMULATEXT and values are known for a legacy cell with a SWITCH returning a range, or nested under a scalar function or operator, and ours matches.

## Context
T-1910 drops the legacy `@` only from a formula-root SWITCH returning array constants (`is_switch_of_array_constants`, static_analysis/legacy.rs); nested or range-returning SWITCH keep the `@` as before. Unprobed: `=SWITCH(1,1,A1:A3)` off and on row, `=SWITCH(1,1,A1:B2)`, `=ABS(SWITCH(1,1,{-2;3}))`, `=-SWITCH(1,1,{2;3})`, `=LEN(SWITCH(1,1,{"ab";"c"}))`. Also `=@SWITCH(1,1,{2;3})` (a typed `@` is assumed to intersect, value 2), several case/result pairs and an array-constant default (`=SWITCH(A1,1,{2;3},{4;5})`) and one-element constants (`{5}`), which T-1910 extrapolates from the probed single pair. Use the excel-probe skill (legacy cells with FORMULATEXT twins).

## Acceptance criteria
- [ ] A probes case covers the shapes above with oracle values; the parser matches them (baseline shrinks or the findings are recorded in references.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/legacy-switch-at-shapes` (57 cells, excel-graph): Excel puts no `@` on a SWITCH in a legacy cell, on its range or one-element results, or on a scalar call over it (`ABS`, `LEN`, unary minus); they spill. The engine matches now: `is_switch_result` skips the `@` on SWITCH results (import and export mirror), SWITCH never takes the call `@`, `legacy_call_reads_a_value` treats a SWITCH argument as no value to intersect; the root-only T-1910 special case is gone. Findings in docs/excel-semantics/references.md. Eval delta: probes +57 cells, all passing, 0 regressions (the slow `union-walk-repeated-in-lambda` case needs `--timeout` above 60 s on this sandbox, unrelated: main takes as long).

## Handoff (resumed)
Sonnet run, escalated after the excel-fidelity reviewer requested changes twice (PR #1294).
- Probe `probes/legacy-switch-at-shapes` (57 cells, oracle sidecar) is good and all 57 cells pass with the current code. Excel: no `@` on SWITCH at the root, on range / one-element / array-constant results, on a 2-D range result, and on ABS/LEN/unary minus over an array SWITCH; a typed `@SWITCH` is kept.
- The code over-extrapolates. Blocking findings of the second review: (1) `Function::Switch` in `new_function` (static_analysis.rs) means a SWITCH never takes the call `@` in any parent, changing binary operators (`SWITCH(..)+0`), N, SUM-like vector args, and moving the `@` onto the caller for INDEX, SIN, ROUND, IF; (2) `is_switch_result` recurses into every result with `add=false`, so operations/calls inside a result (`Z30:Z32*1`, `ABS({-2;3})`, `IF(1,{2;3})`, nested SWITCH) lose their `@` (mirrored in redundant.rs). `legacy_call_reads_a_value` is already narrowed to ABS/LEN with SWITCH as first argument; `arg.ungrouped()` there also extends to `ABS((SWITCH(..)))`, unprobed.
- Next: narrow the code to the probed shapes (root, bare range/array-constant results, ABS/LEN first arg, unary minus) and keep main's behavior elsewhere, or add a second probe for the other shapes and implement Excel's answers. Then fix the UNVERIFIED wording in docs/excel-semantics/references.md (it wrongly says other callers "keep their `@`") and T-2121's Context (add result-operation and N/vector-argument shapes).
- Also: local `make eval-check` showed "new-coverage failures 2, known failures 1666 -> 1667" together with a timeout of the slow case `probes/union-walk-repeated-in-lambda` (67 s on unmodified main too); with `--timeout 300` there were 0 new failures and the baseline gained only the new case. CI is green on the PR head.
- A scratch harness the reviewer used prints the internal `@` form per formula (old vs new); rebuild one from `add_implicit_intersection(.., true)` to compare shapes.
