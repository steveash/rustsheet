---
id: T-3238
title: "Gamma quantile landing point away from p = 0.999: gen-m9 A168 and GAMMA.xlsx F21 visibly off (T-3016 residue)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-3146]
---

## Goal
The two gamma-quantile cells T-0972 made visibly off that T-3016 did not fix reach visible parity (within half a unit
of Excel's 15th significant digit).

## Context
T-3016 fixed shape 1 with the exponential's closed form. Of decision 0152's five cells, two were left; the
`CHISQ.INV(0.999,5)` pair went to T-3146 (its "whichever task lands the landing rule first" note). Both of these are a
few ulps off the true quantile in Excel, so they need Excel's own landing point, not a more accurate kernel:
- gen-m9/ecc87f80077959e0 Calc!A168 `CHISQ.INV.RT(MIN(Data!A9:A12),7)` = `CHISQ.INV.RT(0.3,7)`: Excel
  8.383430828608388, ours 8.383430828608383. The true quantile is 8.3834308286083845 (mpmath, 50 digits): Excel is 2 ulp
  above it, ours 1 ulp below.
- ironcalc-calc/STATISTICAL/GAMMA.xlsx `GAMMA.DIST GAMMA.INV`!F21 `GAMMA.INV(p,9,1E6)`, its p the cell to
  its left, `GAMMA.DIST(10,9,1E6,TRUE)` (p near 2.76E-51, a deep lower tail): Excel 9.999999999999998, ours 10.000000000000004.
  - Part of the gap is upstream. That p is 2.755707120924027E-51 in Excel and 2.7557071209240235E-51 in ours; the true
    value is 2.7557071209240216E-51. That cdf belongs to T-2638 and T-2561.
  - For Excel's p, the true quantile is 10.000000000000002 and Excel is 2 ulp below it.
  - For our p, the true quantile is 10.0000000000000008 and ours is 2 ulp above it. So at shape 9, at x times the rate
    of 1E-5, `quantile_tail`'s lower form (`powf * exp(-x - ln Gamma(a + 1))` times the series) is off by about
    2E-15 relative.

Code: `incomplete_gamma::gamma_quantile`, `quantile_tail`. Evidence on how Excel's landing point moves:
docs/excel-semantics/statistics.md, "Excel's gamma quantiles (T-0972 ...)", and T-3148's probe.

## Acceptance criteria
- [ ] A168 and F21 are within half a unit of Excel's 15th significant digit, with no new baseline line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The p = 0.999 cells (T-3146) and the 34 last-digit cells (T-3239).
- The GAMMA.DIST cdf that feeds F21 (T-2638, T-2561).
