---
id: T-2391
title: "Probe and match YIELD at a price far above par (1E6): #NUM!, not a negative yield"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`YIELD` at a price far above par gives Excel's `#NUM!` where Excel has no root, not a negative yield.

## Context
- Found by the T-2248 smoke check (one formula per financial edge-pool entry, excel-graph, 2026-10-22; the scratch
  suite was deleted, so the probe is this task's to commit). docs/excel-semantics/financial.md "Financial edge-pool
  smoke check (T-2248)" lists the observed values.
- Observed: `YIELD(39448,43831,0.05,1E+6,100,2)` and `YIELD(39448,43831,0.05,1E+300,100,2)` are `#NUM!` in Excel; ours
  -0.6330938491144911 for the first. `YIELD(...,200,100,2)` is -0.02224401627496982 in Excel (ours close).
- Probe a price sweep between 200 and 1E6 (and other coupon counts) to find where Excel's solver gives up: a yield bound or
  an iteration cap. financial.md "YIELD stops short of PRICE's root" and "Yield
  and odd-coupon work cap" describe the solver. Code: functions/financial/financial_bonds.

## Acceptance criteria
- [ ] A probe case sweeps the price to locate the cut, filled through excel-graph.
- [ ] YIELD matches Excel's `#NUM!` cells; the financial.md entry states the rule (VERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's last-digit misses ("YIELD stops short of PRICE's root").
