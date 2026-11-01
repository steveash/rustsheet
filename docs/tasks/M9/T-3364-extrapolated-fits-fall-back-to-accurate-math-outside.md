---
id: T-3364
title: "Extrapolated fits fall back to accurate math outside their measured range: GAMMA.INV Wilson-Hilferty spread, F density Stirling error past df 56, Student-t df bands"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3355, T-3361]
---

## Goal
The fitted pieces that are applied beyond the inputs they were measured on stop at their measured range and fall back to
accurate math outside it (decision 0188 R4.3), and GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV follow the math on
the huge-shape Wilson-Hilferty closed-form branch, where Excel is about 2.7e-4 off (decision 0188 R3); their `#NUM!`
and true-quantile branches keep matching Excel.

## Context
- Owner's 2026-10-31 assessment (decision 0188, code audit): extrapolated fits.
  - statistical/incomplete_gamma/huge_shape.rs `WILSON_HILFERTY_SPREAD` and its terms: fitted at shape 1E13, applied
    from 2E10. Excel's GAMMA.INV at a huge shape is about 2.7e-4 off the math, an R3 case (`MATH-OVER-EXCEL`,
    statistics.md).
  - statistical/f_density.rs `STIRLING_ERROR` table: used past df 56, beyond its measured cells.
  - statistical/student_t.rs df bands (`LOG_GAMMA_BETA_MIN` 6.5, `LOG_GAMMA_BETA_MAX`, `SERIES_MAX_DF`,
    `ASYMPTOTIC_MIN_SHAPE`).
- The `FITTED(T-XXXX, n_cells, range)` tag comes from T-3361; this task uses each site's recorded range.
- T-3354 (P0) may touch the F density first; build on it.
- From T-3350 (2026-10-31), cells on the R3 closed-form branch (ours is Excel's closed form, about 2.66E-4 from the
  math; owned with T-3272 for the digits):
  - probes/t3350-chisq-inv-huge-df A301:A351 (visibly off: A301, A313, A325, A326, A337, A338, A349, A350, A351);
  - probes-numeric/t3350-r4-chisq-inv-sweep A122, A127, A155 (outside both Gate A bounds).

## Acceptance criteria
- [ ] Each piece above applies only inside its recorded `FITTED` range; outside it the code computes the accurate value
      (an mpmath-checked unit test on each side of each boundary).
- [ ] On the closed-form branch (statistics.md's `MATH-OVER-EXCEL` entry; t3070 A110, t2982 A7-A23, and the closed-form numeric cells of the five functions among t3070 A304-A327; A315-A317 are CHISQ.DIST/CHISQ.DIST.RT and stay `#NUM!`)
      the five functions return the math within 1e-9 relative; the `#NUM!` and true-quantile branches keep Excel's
      results.
- [ ] A fresh log-uniform sweep over each affected function's domain (R4.2) shows the Gate A envelope (R2).
- [ ] No gated cell regresses beyond decision 0083's allowance; cells moved to the math by R3 are listed in the Outcome.
- [ ] Before any engine change follows the math, the region's evidence points (oracle-filled, with 50-digit math
      values) and its bounds are committed and the `MATH-OVER-EXCEL` entry cites them; the engine change comes after,
      in the same PR or a later one.
- [ ] The math path keeps directive 5's caps (bounded iterations, no panic); decision 0188 R3 never overrides them.
- [ ] R3 cells are marked as the Gate A spec (T-3355) says. Interim route until T-3359 lands: the cells that leave
      Excel's value are listed in the Outcome, they still count as visibly off in the report, and their baseline
      regression is recorded with a decision record citing decision 0188 R3.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-validating the in-range form selectors (T-3365).
