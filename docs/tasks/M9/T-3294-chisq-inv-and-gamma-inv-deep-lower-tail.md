---
id: T-3294
title: "CHISQ.INV and GAMMA.INV deep lower tail at shape 20 and above: Excel's leading-factor root (CHISQ.INV(1E-300,50) is 2,484 ulp off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`CHISQ.INV(1E-300,50)` (probes-numeric/t1543-inverse-cdf-tiny-p A31) and any other deep lower-tail CHISQ.INV or
GAMMA.INV cell at shape 20 or more reach Excel's value, or the rule that excludes them is recorded.

## Context
T-2561 found that at shapes from 1 to 20, Excel's tiny lower-tail quantile is the root of the cdf's leading factor,
`EXP((LN(p)+GAMMALN(a)+LN(a))/a)`. That is `incomplete_gamma::excel_tiny_quantile`, gated by a cdf check fitted to 10
cells. At shape 25 Excel's `CHISQ.INV(1E-300,50)` = 2.0354283669760313E-11 is that root bit for bit. The engine's
crossing is 2,484 ulp off, visibly off. The T-2561 rule does not cover this cell, for two reasons:
- its shape bound `DIRECT_SHAPE_MAX` excludes shape 25;
- its check rejects the root, because GAMMA.DIST there is 9.8E-12 off p (e^-x times the series, at x = 1E-11).

So Excel's own test for when it takes the root is not the check. The candidates and counts are in
docs/excel-semantics/statistics.md, in the T-2561 entry.

## Acceptance criteria
- [ ] A31 is within half a unit of Excel's 15th digit, or the Outcome records why it cannot be reached.
- [ ] No cell that T-2561's rule makes bit-exact gets worse, and no new known-failure line appears in eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes from 1E9 (T-3070).
