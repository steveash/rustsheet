---
id: T-0276
title: "IFS and SWITCH lift over array/range conditions elementwise (probe ifs-switch-array-values)"
milestone: M2
lane: functions-logic
status: in-progress
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
- [ ] The probe cases for array conditions / expression pass (check `make eval-show CASE=ifs-switch-array-values`).
- [ ] Unit tests for IFS and SWITCH broadcasting, including value-argument ranges.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
