---
id: T-3505
title: "GAMMA.INV returns #NUM! at shapes near 8E6 where Excel returns a number"
milestone: M9
lane: functions-stat
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV return Excel's number, not `#NUM!`, at the shapes below the
huge-shape band (1E9, T-3070) where the engine gives up today.

## Context
- Held-out blind sweep (excel-graph, filled 2026-10-31): `GAMMA.INV(0.5,7990464.81598859,1)` is #NUM! in the engine
  and 7990464.482655259 in Excel. A wrong type is a semantic failure. The other 135 GAMMA.INV cells of the sweep at
  shapes 20 to 1E9 with p from 0.01 to 0.5 were fine.
- The shape lies below `HUGE_SHAPE_MIN` (incomplete_gamma/huge_shape.rs), so the cause is in `gamma_quantile`
  (incomplete_gamma/mod.rs): suspects are the one `MAX_TERMS` term budget its cdfs share, the bracket, or another
  directive-5 cap.
- Root cause: `gamma_quantile` gives all the cdfs of one inversion a single budget of `MAX_TERMS` (2^19) terms
  (directive 5). Where the smaller tail's crossing lies below x = a + 1 the cdf is the power series, about 7.3 sqrt(a)
  terms near the median, and the refinement from the Wilson-Hilferty start takes about 25 cdfs (its first step,
  2^-30 of the start, spans about 2^23 doubles, so about 23 halvings). From about 7E6 that passes 2^19 terms, the
  inversion is NaN and the cell `#NUM!`. From 1E8 every cdf is Temme's (one term), so the band ends there. Where the
  crossing lies above a + 1 (the upper tail is the smaller one) the cdf is the continued fraction, which is cheap.
- Failing region on main 3003a898b (local sweep, release build: 81 shapes log-spaced from 1E5 to 1E9 by 14 p from
  1E-300 to 1-1E-10, GAMMA.INV(p,a,1), CHISQ.INV(p,2a), CHISQ.INV.RT(p,2a); 3,402 cells), all `#NUM!`:
  - shape from 7.94E6 (7.08E6 still a number) up to 8.91E7 (1E8 a number), and only where the lower tail is the
    smaller one: GAMMA.INV and CHISQ.INV with p <= 0.5, CHISQ.INV.RT and CHIINV with p >= 0.5;
  - the lower shape edge moves up as the quantile moves into the tail (the series needs fewer terms there):
    p = 0.5 from 7.9E6, 0.3 from 1.0E7, 0.1 from 1.1E7, 0.01 and 0.001 from 1.6E7, 1E-10 from 3.5E7, 1E-30 from
    7.9E7; never at 1E-100 or 1E-300 below 1E8. For CHISQ.INV.RT the mirror: 0.7 from df 2E7, 0.9 from 2.2E7, 0.99
    from 3.2E7, 0.999999 from 5E7, 1-1E-10 from 7.1E7;
  - GAMMAINV and CHIINV are the same kernel (GAMMA.INV and CHISQ.INV.RT) and fail at the same points
    (probes/t3505-gamma-inv-budget-band).
  - The scale does not matter (the search is at shape a, x times the rate).
- Overlap: T-3350 (PR #2597, approved and marked ready for the merge train on 2026-10-31) fixes the same root cause
  from the CHISQ.INV side: from shape 1E5 the quantile's cdfs are Temme's, one term each (`QUANTILE_TEMME_SHAPE_MIN`),
  and it retires the DEVIATION in statistics.md "Incomplete gamma: per-call cap and lifting".

## Acceptance criteria
- [x] The failing region is measured and recorded (shape and p ranges, per function).
- [x] A probe case under eval/suites/probes/cases/ (tag M9), filled by the oracle (excel-graph), covers the region and
      points on both sides of it.
- [ ] Every cell of that case has Excel's type, and its numbers are within 15 significant digits of Excel's (decision
      0083's rule), or the residue is listed.
- [ ] Per-call work stays bounded (directive 5): no cap removed.
- [ ] The finding is recorded in docs/excel-semantics/statistics.md.
- [ ] No gated regression and no cell newly visibly off (decisions 0083, 0111); main-vs-HEAD per-cell comparison in
      the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The huge-shape band from 1E9 (T-3070) and the last-digit work of T-3269, T-3285 and T-3311.
