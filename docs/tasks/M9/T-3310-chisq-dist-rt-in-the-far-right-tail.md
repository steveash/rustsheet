---
id: T-3310
title: "CHISQ.DIST.RT in the far right tail: Excel's Q from 1E-20 to 1E-160 is up to 1E-14 relative off ours (273 visibly-off t2561 probe cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
CHISQ.DIST.RT and the density CHISQ.DIST(x,df,FALSE) match Excel to half a unit of the 15th digit in the far right
tail: x from 96 to 815, df 3 to 22, p from 1E-20 to 1E-160.

## Context
T-2561 added probes-numeric/t2561-chisq-dist-rt-past-edge, 160 rows, excel-graph. Each row starts at the largest
CHISQ.INV.RT quantile Excel returns for a df and steps up to 7% past it. On main after T-2561 the case has 273
visibly-off cells; none was in eval/ before. Ours is up to 5.6E-14 relative off Excel, with both signs, in both
columns:
- A1 `CHISQ.DIST.RT(96.239124,3)` is 9.999999693620438E-21 in Excel against ours 9.999999693620467E-21;
- A141 `CHISQ.DIST.RT(719.864103,20)` is 5.6E-14 off;
- B134 `CHISQ.DIST(693.123446,19,FALSE)` is -4.5E-14 off.

The density is off as much as the tail, so the shared prefactor is the likely cause. Here x/2 is 15 to 40 times the
shape, which is Excel's ratio form in `gamma_prefactor` (T-2638), or the shape-below-20 direct form. Fit it against
this case, using the excel-probe skill for more x if needed.

Three quantile cells in the T-2561 probes are visibly off for the same kind of reason:
- t2561-chisq-inv-rt-num-edge A72 `CHISQ.INV.RT(1E-116,17)`;
- t2561-chisq-inv-rt-num-large-df A210 `(1E-300,43)`;
- t2561-chisq-inv-rt-num-large-df A280 `(1E-300,50)`.

Check them after the cdf fit.

## Acceptance criteria
- [ ] The 273 visibly-off cells of t2561-chisq-dist-rt-past-edge are within half a unit of Excel's 15th digit, or
      the ones left are named in the Outcome, with the forms tried and their counts (decision 0056).
- [ ] No new known-failure line in any eval/baseline file (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The #NUM! edge of CHISQ.INV.RT (T-2561 tabulates it).
