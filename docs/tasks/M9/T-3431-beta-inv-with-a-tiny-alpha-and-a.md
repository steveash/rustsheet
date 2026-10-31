---
id: T-3431
title: "BETA.INV with a tiny alpha and a large beta near p = 1 is grossly off: the upper-tail incomplete beta returns 0 or 1"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
BETA.INV returns Excel's value (on the math) for p near 1 with a tiny alpha and a large beta, where the engine is off by
orders of magnitude today.

## Context
- Found by T-3351's fresh sweep (probes-numeric/t3351-beta-inv-sweep A200, excel-graph):
  `BETA.INV(1-1.10491974750915E-14,0.0050070029475036,171.171095424379)` is 0.12939405629301814 in Excel, which is
  0.129394056293018177... in 50-digit mpmath. Ours is 2.925059559426143E-5.
- Cause: `incomplete_beta::beta_reg_upper(t, 1 - t, a, b)` with a = 0.005 and b = 171 returns exactly 1 at
  t = 2.9E-5 and exactly 0 at t = 0.1294. The true upper tail is about 0.05 at 2.9E-5 and 1.1E-14 at 0.1294. The
  refinement in `beta.rs` `beta_quantile` crosses that broken cdf, moving statrs' correct start (0.12939405629301814)
  to 2.9E-5. `beta_lower_crossing(b, a, 1 - p)` (bratio) gives the same wrong value, so check BETA.DIST's lower tail
  at these shapes too.
- Related: T-3351 (the quantile that rounds to 1), T-2119 (the refinement on the T-0970 incomplete beta).

## Acceptance criteria
- [ ] A probe (probes, M9, excel-graph) with BETA.INV and BETA.DIST at alpha 1E-3 to 0.05, beta 10 to 1E4, and p or
      x across the upper tail, reproduces the bug before the fix (cells listed here).
- [ ] After the fix those cells are within 15 digits of Excel, with no gated regression. Sweep A200 passes the
      Gate A envelope.
- [ ] The rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The quantile that rounds to 1 (T-3351, T-3430).
