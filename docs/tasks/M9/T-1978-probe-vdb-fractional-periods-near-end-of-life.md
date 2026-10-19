---
id: T-1978
title: "Probe VDB fractional periods near end of life and with salvage above cost"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VDB's fractional-period and salvage-above-cost rules rest on oracle evidence beyond single cells.

## Context
T-1764 fixed probes/fin-vdb-modes A9 (unclipped prorated charge, total capped at cost-salvage, salvage <= cost only) and A17 (no straight-line switch when salvage > cost) from one cell each. Probe with the oracle (excel-probe skill): VDB(2400,3000,10,0,0.5), (0,1.5), (1,1.5), (1,2), the same with no_switch TRUE, salvage = cost, VDB(2400,300,10,8.5,9.5,2,TRUE), (9.25,9.75), VDB(2400,300,9.5,8,9.5), (9.5,10) with factor 3; and AMORDEGRC rates 1/4.5, 1/3, 1/3.5 (life band edges). Then correct financial_depreciation.rs and amortization.rs.

## Acceptance criteria
- [ ] probe cases with oracle values committed; the implementation matches them
- [ ] `make check` green; reviewers approve.

## Out of scope
-
