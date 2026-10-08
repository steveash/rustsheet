---
id: T-0276
title: "IFS and SWITCH lift over array/range conditions elementwise (probe ifs-switch-array-values)"
milestone: M2
lane: functions-logic
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
IFS and SWITCH evaluate element by element when conditions or the SWITCH expression are arrays or ranges, as in Excel.

## Context
T-0272 probe `eval/suites/probes/cases/ifs-switch-array-values` (oracle-filled): `IFS({TRUE,FALSE},1,TRUE,2)` spills 1,2; `IFS(A1:B2>1,5)` is `#N/A,5;5,5`; `SWITCH({1,2},1,"a",2,"b")` is a,b; `SWITCH(A1:B2,1,"a","z")` is a,z;z,z. `fn_ifs` (`functions/logical/mod.rs`) and `fn_switch` (`logical/switch.rs`) take the scalar path (`TODO add implicit intersection`). Static analysis is already `Unknown` (T-0272); this task is the runtime lift, modelled on `if_impl`.

## Acceptance criteria
- [x] The probe cases for array conditions / expression pass (check `make eval-show CASE=ifs-switch-array-values`).
- [x] Unit tests for IFS and SWITCH broadcasting, including value-argument ranges.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `fn_ifs` lifts from the first array or range condition, and `fn_switch` lifts over an array or
  range expression. Both go through the existing `lift_with_cost` (broadcast, size and visit caps)
  with per-element helpers (`ifs_element`, `switch_element`). Scalar conditions and expressions keep
  their path, so a range value stays a reference (`SUM(IFS(TRUE,A1:B2))` is 10).
- probes/ifs-switch-array-values passes 41/41. New unit tests are in test/test_ifs_switch_lift.rs.
  A new probe, probes/ifs-switch-lift-edges (excel-graph, 26 cells), confirms the edges the engine
  chose: value broadcasting, scalar short-circuit, single-cell ranges, ragged shapes and an array
  case list. The engine matches all of them, and logic.md records them.
- Eval delta: probes known failures 363 -> 351 (fixed 12; new coverage 0; regressions 0).
