---
id: T-1871
title: "F.INV and BETA.INV give Excel's values where statrs' inversion saturates, panics or never finishes"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
F.INV (and BETA.INV where its shapes are extreme) gives Excel's value at a tiny probability where the statrs inversion
the engine uses saturates, panicked or never finished, instead of `#NUM!` or a wrong number.

## Context
Found by T-1848, which capped statrs' `inv_beta_reg` (`statistical/inverse_beta.rs`) so that it is `#NUM!` where statrs
panicked or hung (statistics.md "Quantile searches: per-call cap and lifting", a DEVIATION). Excel's values,
probes/stat-quantile-lift (excel-graph, 2026-10-18):
- E1-E5, E12: `F.INV(p,5,5)` follows the power law 5.081599921802649E-09 * (p/1E-20)^0.4: `F.INV(1E-300,5,5)` =
  5.081599884913164E-121 (ours `#NUM!`, statrs panicked), `F.INV(1E-170,5,5)` = 5.081599884913239E-69 (ours `#NUM!`),
  `F.INV(1E-160,5,5)` = 5.081599884913189E-65 and `F.INV(1E-100,5,5)` = 5.081599884913108E-41 (ours, as on main,
  1.3470392369521052E-15: statrs saturates).
- E6: `F.INV(1E-50,6000,4)` = 0.0004924127886173762 (ours `#NUM!`, statrs never finished).
- E9: `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` = 0.0691959487211844 (ours `#NUM!`, statrs never
  finished); the true quantile is about 0.003, so this is Excel's erratic extreme-shape behaviour (statistics.md
  "Inverse CDF functions at a tiny or near-1 probability (T-1543)").
A lower-tail form of the F quantile (the incomplete beta's leading term, as `t_dist::t_tail_quantile` does for Student's
t) or a bisection fallback on [0, 1] (`special::bisect_unit_interval`) are candidates; T-1745 found Excel's BETA.INV
solves its own cdf to the crossing. Keep the per-call caps and the lift weights of kinds.rs `QUANTILE_WEIGHT`.

## Acceptance criteria
- [ ] probes/stat-quantile-lift E1-E6 pass or are within 4 ulp (named in the Outcome); E9 matches or its rule is
  recorded as UNVERIFIED in docs/excel-semantics/statistics.md.
- [ ] The values probes-numeric and ironcalc-calc check for F.INV and BETA.INV are unchanged or better; the DEVIATION
  entry shrinks to what remains.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
