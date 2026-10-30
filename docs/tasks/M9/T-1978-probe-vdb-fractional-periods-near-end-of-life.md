---
id: T-1978
title: "Probe VDB fractional periods near end of life and with salvage above cost"
milestone: M9
lane: functions-financial
status: done
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
- [x] probe cases with oracle values committed; the implementation matches them
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/fin-vdb-amordegrc-edges (143 cells, excel-graph): all 59 VDB cells (salvage above and equal to cost in both
modes, fractional windows near the end of life, fractional life 9.5 with factor 2 and 3) already matched the T-2185
model, so financial_depreciation.rs is unchanged. AMORDEGRC refuses only a life `1/rate` <= 2: lives 2.000001-3 and
4.01-4.99 return numbers (coefficients 1.5 and 2 by `ceil(1/rate)`), so `asset_life_refused` drops the [2, 3] and
(4, 5) bands and `amor_depr_coeff` its unreachable 1.0 branch. 20 cells were wrong before the fix; 143/143 pass now.
Recorded in docs/excel-semantics/financial.md "AMORDEGRC life bands" and "VDB charge order".
