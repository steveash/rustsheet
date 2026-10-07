---
id: T-1038
title: "AGGREGATE lifts an array function_num, option or k"
milestone: M1
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
AGGREGATE with an array function_num, option or k gives Excel's values.

## Context
Found by T-0993. Excel gives 23 for `AGGREGATE({9,4},0,E1:E6)`, `#DIV/0!` for `AGGREGATE(9,{0,4},D1:D11)` and 10 for `AGGREGATE(14,6,D1:D11,{1,2})` in the checked cell; the engine gives `#N/IMPL!` (probes/aggregate-args Misc!A36:A38, known failures). Probe whether these spill (check the neighbouring cell) before choosing lift versus first element. Code: `fn_aggregate` (crates/rustsheet-engine/src/functions/aggregate.rs), docs/excel-semantics/numbers.md "AGGREGATE". Static analysis gives AGGREGATE `args_signature_npv` (options and k marked Vector), which lifting must revisit.

## Acceptance criteria
- [x] probes/aggregate-args Misc!A36:A38 pass, and the spill shape (if any) is probed and matched.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
AGGREGATE lifts an array function_num, options or fourth argument element-wise and spills.
Each element is a scalar AGGREGATE call, broadcast like the element-wise operators, with `#N/A`
past a shorter array. The lift reuses `Model::lift_criteria` (its cell-visit cap included), and
`aggregate_static_result` gives the spill shape. probes/aggregate-array-args (48 cells, new)
pins the shapes, broadcasting, k in the reference form, per-element form and invalid elements.
Eval: probes known failures 210 -> 207 (aggregate-args Misc!A36:A38 fixed), new case 48/48,
no regressions. Review follow-up: the shared value arguments are evaluated once per call, and
the visit cap counts every reference and computed array (unit test
`lifted_calls_over_large_arrays_hit_the_visit_cap`). Follow-up: T-1058 (range function_num/options).
