---
id: T-3252
title: "GAMMA.INV at a shape below 1 and a quantile from 1E-21 up: Excel's Newton steps from its closed form"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3013]
campaign: visible
---

## Goal
GAMMA.INV and GAMMAINV at a shape below 1 (not 1/2) give Excel's value to 15 digits where the quantile at scale 1
is 1E-21 or more, above the region T-3013 matched.

## Context
- T-3013 (docs/excel-semantics/statistics.md, T-3013 bullets under "Excel's CHISQ.INV and GAMMA.INV are exact in the
  lower tail"): below a unit quantile of 1E-21 Excel is the closed form `EXP((LN(p)+GAMMALN(a)+LN(a))/a)`, plus one
  Newton step on its own GAMMA.DIST cdf where that step is over 3.5E-14 of x. Above it the engine keeps its crossing
  search.
- Cells still off there: probes/t3012-gamma-inv-small-shape A18, A29, A30, A34, A35, A36 and B18, B29, B30, B34,
  B35, B36 (a = 0.25, 0.75, 0.9; B29, A35 and B35 visibly off), and gen-m9/60b0b6e69ad35a45 A210
  `GAMMAINV(0.5,MINVERSE(INT(60)),1.5E+10)` (shape 1/60, unit quantile 4.9E-19; Excel 7.404834485321526E-09,
  ours 83 ulps low). On the engine's model of Excel's cdf, one step from the closed form lands 1 to 34 ulps off
  Excel, and later steps wander by tens of ulps. A30 is reached at the fourth step. The cdf model has a 1-ulp residue
  above a·LN(x) = -16 (T-3242), which the step magnifies by 1/a, so that residue may need fixing first.
- Also open: at shape 1/2, column B (scale 1E+15) is 1 to 4 ulps off where Excel is column A times the scale. A
  unit-quantile-times-scale rule fixes those cells but loses three bit-exact cells elsewhere (statistics.md).
- Engine: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs `excel_small_shape_quantile`,
  `CLOSED_FORM_QUANTILE_MAX`; gamma.rs `gamma_inv_of`.

## Acceptance criteria
- [ ] The visibly-off cells above (B29, A35, B35, gen-m9 A210) are within half a unit of the 15th digit of
      Excel's value, with no lost bit-exact cell in any suite.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md, citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes of 1 or more (T-0972, T-2367, T-3016).
- Bit-level residue within 15 digits: a `defer: phase-1b` follow-up.
