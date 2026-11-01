---
id: T-3618
title: "POISSON.DIST cdf off Excel near the mean (mean 0.5-1.4 times k+1, k above 20) on the T-3509 blind probe"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The cumulative POISSON.DIST with the mean from 0.5 to 1.4 times k + 1 at k above 20 matches Excel within half a unit
of the 15th significant digit, or the Outcome lists the forms tried with their counts (decision 0056).

## Context
- Found by the T-3509 held-out probe (docs/research/t3509/README.md, "POISSON cdf near the mean" row): 5 of the 35
  probe cells with k + 1 at 20 and above and the mean 0.5 to 1.4 times k + 1 are visibly off; the other 30 are within
  15 digits. The cdf is Q(k + 1, mean), the gamma upper tail at shape k + 1, so this shares code with CHISQ.DIST.RT.
- Excel against the 50-digit math: 20 ulp (median), up to 175 ulp (A355). Ours is 39 ulp (median) from the math and
  closer than Excel at 1 of 5. All 5 are inside Gate A.
- Example: `POISSON.DIST(2105,2460.67714424429,TRUE)` Excel 1.0667637123053853E-13, ours 1.0667637123053791E-13,
  math 1.0667637123053887E-13.
- Neighbours: T-3269 is POISSON's deep upper tail (mean 1.4 times k + 1 and up); T-3522 is the pmf at k above 1000;
  T-3286 is CHISQ.DIST.RT near the mode at shapes 21 to 1E8 (the `NEAR_MODE` band). Check first whether these cells
  are in T-3286's band; if so, fold them into T-3286 and close this task as a duplicate.
- Code: `statistical/poisson.rs`, `statistical/incomplete_gamma/`.
- Cells (excel-graph, 2026-10-31), probes-numeric/t3509-blind-poisson-dist.case.json, sheet S (5): A142, A397, A29,
  A355, A418.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns is within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up. The probe cells are evidence for the region, not a fitting set: probe the band first
(the `excel-probe` skill).

## Acceptance criteria
- [ ] The 5 listed cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms tried with
      their counts, or the cells move to T-3286 with this task closed as its duplicate.
- [ ] Decision 0188 R4 guards: any candidate is chosen on a hold-out split of its own evidence (R4.1, train and test
      counts side by side, accepted only if it beats the correctly rounded candidate on test); it is never fitted to
      the T-3509 probe, which is scored before and after and both per-function tables go in the Outcome
      (docs/research/t3509/README.md); a fresh log-uniform sweep is filled after the model is frozen (R4.2); every fitted
      constant, cut-point or table carries `FITTED(T-XXXX, n cells, range)` and falls back to accurate math outside
      that range (R4.3); nothing is fitted below the 4-ulp oracle floor (R4.4); the Gate A envelope result is reported
      (R4.5).
- [ ] No gated cell regresses; no new known-failure line in eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- POISSON's deep upper tail (T-3269) and pmf (T-3522, T-3295, T-3247).
