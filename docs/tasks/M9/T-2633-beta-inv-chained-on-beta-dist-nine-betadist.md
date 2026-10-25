---
id: T-2633
title: "BETA.INV chained on BETA.DIST: nine BETADIST_BETAINV.xlsx cells 1-4 ulp after T-2119"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The nine `BETA.INV(BETA.DIST(...))` cells that T-2119 moved from bit-exact to 1-4 ulp match Excel to the bit again,
without moving BETA.DIST's cdf off `incomplete_beta`.

## Context
T-2119 put BETA.DIST's cdf on `incomplete_beta::beta_reg_complement` and refined BETA.INV to the crossing of that cdf
(`special::refine_increasing`). That fixed 27 gated lines, and recorded these regressions under decision 0083's
standing allowance (all numeric, none visibly off). Excel against ours in `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx`
`Sheet1`:
- H2: 3.0 vs 3.000000000000001 (2 ulp)
- H23: -0.23399999999999987 vs -0.23399999999999999 (4 ulp)
- H42, H44, H46, H47, H48, H49: 0.23399999999999999 vs 0.234 (1 ulp)
- H67: 3.0 vs 2.999999999999999 (2 ulp)

These cells passed before only because statrs' cdf and its inverse were wrong in the same way (T-2119 Context). The
H42 group follows G42 (BETA.DIST), which is still 4 ulp from Excel. So fitting G42's last bits probably moves H42-H49
too. Code: `functions/statistical/beta.rs`, `inverse_beta.rs`, `special::refine_increasing`. See
docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)".

## Acceptance criteria
- [ ] The nine lines above leave `eval/baseline/ironcalc-calc.txt`, with no new known-failure line in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETADIST_BETAINV G5 and G26 (the T-1044 sweep).
