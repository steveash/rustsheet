---
id: T-2075
title: "ISREF of a LAMBDA call whose body is an IF picking a reference XLOOKUP is TRUE"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`ISREF(LAMBDA(x,IF(TRUE,XLOOKUP(1,J1:J2,H1:I2)))(1))` is TRUE, as Excel gives, without changing
what GROUPBY and PIVOTBY see from a LAMBDA that returns a reference.

## Context
Found in T-2071. probes/take-drop-over-ifs-switch-picks A78 (excel-graph, 2026-10-19; H1:I2 =
3,6;4,8, J1:J2 = 1,2) is TRUE; the engine gives FALSE. `ISREF(LAMBDA(x,x)(XLOOKUP(..)))` is
already TRUE (A74). A LAMBDA call in reference context reads its body with
`evaluate_reference_arg` (`LambdaCaller::keeping_references`, functions/logical/lambda.rs), which
does not follow the arm an IF picks into a reference-returning XLOOKUP the way
`evaluate_picked_reference_arg` (functions/util.rs) does. GROUPBY and PIVOTBY share that caller
(probes/groupby-lambda-forms, groupby-pivotby-functions), so probe a GROUPBY whose LAMBDA returns
`IF(TRUE,XLOOKUP(..))` before switching the read, and check SUM/SUBTOTAL over the same call.

## Acceptance criteria
- [ ] probes/take-drop-over-ifs-switch-picks passes in full.
- [ ] GROUPBY/PIVOTBY and aggregate behavior over such a LAMBDA is probed and matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
