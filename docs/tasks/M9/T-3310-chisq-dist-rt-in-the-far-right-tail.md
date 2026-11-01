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
campaign: visible
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
- [ ] The CHISQ.INV.RT `#NUM!` edge (chisq.rs `RIGHT_TAIL_P_MIN`) is probed further. T-2561 inferred CHIINV and the
      in-decade p, so the probe has: CHIINV at the edge decade of df 3, 10, 22 and 42; a mantissa sweep inside one
      decade (p = m E-k, m = 1 to 9.9) at df 3, 10 and 30; and df 51 to 100 at p 1E-250 to 1E-300. The table
      either matches it or is corrected, with the CHIINV and in-decade notes in chisq.rs and statistics.md
      updated.
- [ ] No new known-failure line in any eval/baseline file (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value;
bit-level misses left then go to a `defer: phase-1b` follow-up.
The campaign is `visible`, not `accuracy`. Checked in T-2561 review round 1 against mpmath at 60 digits,
Excel's own far-tail values are what is inaccurate: CHISQ.DIST.RT and the density in this probe are up to 7.2E-14
relative from the reference, with 115 of 160 cells in each column (CHISQ.DIST.RT and density; 230 of 320 in all) more than 5E-15 off. So the work is reproducing Excel's far-tail
prefactor, not a more accurate kernel.

## Out of scope
- Refitting the #NUM! edge beyond what the probe items above show.
