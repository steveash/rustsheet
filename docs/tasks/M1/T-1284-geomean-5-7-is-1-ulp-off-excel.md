---
id: T-1284
title: "GEOMEAN(5,7) is 1 ulp off Excel (5.916079783099616, engine ...615)"
milestone: M1
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`GEOMEAN(5,7)` returns Excel's exact value 5.916079783099616.

## Context
`=GEOMEAN(5,7)` is 5.916079783099616 in Excel (probes/subtotal-reference-operands-d H28; excel-graph) and the engine gives 5.916079783099615. Look at `functions/statistical/geomean.rs` for how the product/log is accumulated and find the order of operations that reproduces Excel's bits across the existing GEOMEAN cases.

## Acceptance criteria
- [ ] `GEOMEAN(5,7)` matches Excel bit-exactly, with a probe case added (or the -d case extended) as evidence and no existing GEOMEAN case regressing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
