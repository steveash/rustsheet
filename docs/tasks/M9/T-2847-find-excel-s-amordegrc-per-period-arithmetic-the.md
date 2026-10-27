---
id: T-2847
title: "Find Excel's AMORDEGRC per-period arithmetic: the walk drifts a few ulps from main's from about 10^3 periods (amordegrc-walk-drift, -ties)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2826]
campaign: semantics
---

## Goal
AMORDEGRC's walk reproduces Excel's per-period arithmetic bit for bit, so the walk-drift cells of
probes/amordegrc-walk-drift (D16, E5-E45) and probes/amordegrc-walk-ties (A51, A53, C5, C8, C9) have Excel's value, or
financial.md "AMORDEGRC rounds its charge like ROUND" records why that arithmetic cannot be found from probes.

## Context
From T-2826 (docs/excel-semantics/financial.md "AMORDEGRC rounds its charge like ROUND"):
- The final rounding is ROUND(x,0)'s (15-digit cut of the exact value, then half away): every cell at 1 and 2 periods
  within one ulp of a cut boundary matches main's walk (amordegrc-walk-drift D1-D12, -ties A1-A12, C1-C3, C7).
- Past that, Excel's walk drifts from main's (`remain -= rate * remain`) by a few ulps, in both directions: 1 of 6
  near-boundary cells differs at 10^3 periods (-drift D16), 2 of 6 at 10^4, about half at 5x10^4 to 10^6.
- Not Excel's: an FMA update, `remain * (1 - rate)`, the rate times the coefficient in another order, a charge cut to
  8-16 significant digits or rounded to a whole number before it is subtracted, an accumulated depreciation
  subtracted from the cost (each scored against the 157 cells in a scratch fit; main's walk scores best, 127).
- Ideas left: the first charge or the rate computed another way (1/life, the year fraction first), an x87 double
  rounding (80-bit product rounded to double), a period counter that skips or repeats a period, the cost read
  as `cost - salvage` with the salvage added back. A probe at 1000-3000 periods with many cells near a boundary
  localises the first divergent period cheaply.
Code: functions/financial/amortization.rs (`DegressiveWalk::charge`, `amor_round`).

## Acceptance criteria
- [ ] The listed cells pass (baseline lines removed) with no new failures, or the entry records the arithmetic
  ruled out and why the rest cannot be settled.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AMORDEGRC past 2^20 periods (the closed form, a work-cap DEVIATION).
