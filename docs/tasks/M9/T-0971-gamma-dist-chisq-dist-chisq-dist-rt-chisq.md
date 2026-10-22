---
id: T-0971
title: "GAMMA.DIST/CHISQ.DIST/CHISQ.DIST.RT/CHISQ.TEST: accurate regularized incomplete gamma (43 known failures)"
milestone: M9
lane: functions-stat
status: blocked
priority: P1
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
One accurate regularized incomplete gamma P(a,x) / Q(a,x) pair (Q computed directly, not as 1 - P)
in the engine, used by GAMMA.DIST, CHISQ.DIST, CHISQ.DIST.RT and CHISQ.TEST, replacing statrs'
`gamma_lr`/`gamma_ur`; the gamma density from the same building blocks.

## Context
Kernel map (T-1522, decision 0056): rank 7 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric cases `m9-gamma-dist-a*`, `m9-chisq-dist`, `m9-chisq-dist-rt`, `m9-gamma-inv`, `m9-chisq-inv`, `m9-chisq-inv-rt` (T-1045).
Triage T-0903. Kernel: `statrs::function::gamma::{gamma_lr, gamma_ur}` (series / continued fraction
with statrs' Lanczos `ln_gamma` in the prefactor), reached through `statrs::distribution::Gamma::cdf`/
`pdf` (`functions/statistical/gamma.rs` `fn_gamma_dist`) and `ChiSquared` (`functions/statistical/chisq.rs`
`fn_chisq_dist`, `fn_chisq_dist_rt` (`sf`), `fn_chisq_test` (`1.0 - dist.cdf(chi2)`)).
43 baseline lines in `eval/baseline/ironcalc-calc.txt` (34 NumericClose, 9 NumericUlp):
- `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` `BETA_DIST`: L2, L3(C), L4(C), L5(C), L6(C)
- `ironcalc-calc/STATISTICAL/CHISQ.xlsx` `Sheet1`: G2, G3(C), E4(C), G4(C)
- `ironcalc-calc/STATISTICAL/CHISQ_TEST.xlsx` `Sheet1`: E1
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV`: E2(C), E3(C), E4, E6, E7, E8(C), E9, E10(C), E11(C), E16(C), E19(C), E21(C), E23, E24(C), E34(C), E38(C), E39, E40(C), E41(C), E42(C), E43(C), E44(C), E45(C), E46(C), E47(C), E50(C), E51(C), E53(C), E58(C), E60(C), E61(C), E62(C), E63(C)
(C = NumericClose, unmarked = NumericUlp. 17 of these lines (16 in GAMMA.xlsx plus BETA_GAMMA L5) are
`GAMMA.DIST(10,9,2,TRUE)`, so one fix moves many lines.)

Also in probes (scalar cells beside the T-1872 lift cells, unchanged by it): probes/stat-argument-modes H132
`CHIDIST(1.5,5)` and H136 `CHISQ.DIST.RT(1.5,5)` expected 0.9130698145443954, actual 0.9130698145443957 (NumericUlp);
H152 `CHISQ.DIST(1.5,5,TRUE)` expected 0.08693018545560456, actual 0.0869301854556043 (NumericClose).

Examples (expected vs actual):
- `GAMMA.DIST(10,9,2,TRUE)` E2 (17 lines with this value): expected 0.06809363472184855, actual 0.06809363472184732 (1.8e-14).
- `GAMMA.DIST(1,1,1,TRUE)` BETA_GAMMA L2 / GAMMA.xlsx E4, E6: expected 0.6321205588285577 (= 1 - exp(-1)
  and -expm1(-1) in doubles), actual 0.6321205588285581.
- GAMMA.xlsx E16: expected 5.38226469068969e-54, actual 5.38226469068958e-54; E21 (beta = 1e6): expected
  2.755707120924027e-51, actual 2.7557071209239665e-51.
- `CHISQ.DIST.RT` CHISQ.xlsx G3: expected 0.04601170568923137, actual 0.04601170568923126;
  `CHISQ.TEST` E1: expected 0.3666353840752714, actual 0.3666353840752712.

Hypothesis (the oracle is the judge): Excel uses an accurate incomplete gamma (e.g. Cephes
`igam`/`igamc`, or DiDonato-Morris TOMS 654 with Temme's expansion for large a), with prefactor
`exp(a ln x - x - lgamma(a))` from an accurate lgamma (`special::ln_gamma`), and computes the upper
tail directly for CHISQ.DIST.RT/CHISQ.TEST. a = 1 must reduce to -expm1(-x). The density: try
`exp((a-1) ln x - x/b - lgamma(a)) / b` against `x^(a-1) e^(-x/b) / (b^a Gamma(a))` and keep what matches.
Implement once in `functions/statistical/special.rs`; T-0972 (GAMMA.INV) and possibly T-0927
(POISSON.DIST CDF) reuse it.

## Acceptance criteria
- [ ] No line listed above remains NumericClose (`rustsheet-eval show`), at least 25 of the 43 lines
      leave `eval/baseline/ironcalc-calc.txt`, no new lines in any baseline (POISSON.DIST must not
      regress if it switches kernel; EXPON.DIST stays fixed).
- [ ] Remaining lines (<= 4 ulp) listed in the PR with expected/actual; they wait for the T-1045
      incomplete-gamma sweep (family 6).
- [ ] CHISQ.TEST and CHISQ.DIST.RT use the direct upper tail (no `1.0 - cdf`).
- [ ] Unit tests: P(1,x) = -expm1(-x), P(a,x) + Q(a,x) = 1 within 2 ulp at a few points, a in {0.5,
      9, 500}, huge scale (b = 1e6); bounded iteration counts; `make check` green; reviewers approve.

## Out of scope
GAMMA.INV/CHISQ.INV (T-0972); GAMMA/GAMMALN (T-0973); legacy GAMMADIST/CHIDIST/CHITEST evidence (T-1047, T-1048).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 256 in total.
- GAMMA.DIST: 110 cells (13-14 digits right 84, 10-12 24, 6-9 0, fewer or 0 vs nonzero 2); most in `probes-numeric/m9-gamma-dist-a9.case.json` 27; `ironcalc-calc/STATISTICAL/GAMMA.xlsx` 26. e.g. `GAMMA.DIST(9,9,2,TRUE)` Excel 0.04025731248203771, ours 0.04025731248203709
- GAMMADIST: 6 cells (13-14 digits right 5, 10-12 1, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1048-gammadist.case.json` 6. e.g. `GAMMADIST(10,5,2,TRUE)` Excel 0.5595067149347877, ours 0.5595067149347852
- CHISQ.DIST: 57 cells (13-14 digits right 43, 10-12 13, 6-9 0, fewer or 0 vs nonzero 1); most in `probes-numeric/m9-chisq-dist.case.json` 35; `probes-numeric/t1049-chisqdist.case.json` 19. e.g. `CHISQ.DIST(3,3,TRUE)` Excel 0.6083748237289112, ours 0.6083748237289098
- CHISQ.DIST.RT: 38 cells (13-14 digits right 33, 10-12 4, 6-9 0, fewer or 0 vs nonzero 1); most in `probes-numeric/m9-chisq-dist-rt.case.json` 15; `probes-numeric/m9-legacy-chidist.case.json` 10. e.g. `CHISQ.DIST.RT(3,3)` Excel 0.39162517627108884, ours 0.39162517627109017
- CHIDIST: 10 cells (13-14 digits right 10, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-chidist.case.json` 10. e.g. `CHIDIST(10,2)` Excel 0.006737946999085467, ours 0.006737946999085472
- CHISQ.TEST: 2 cells (13-14 digits right 2, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-chitest.case.json` 1; `probes-numeric/t1049-chisqtest.case.json` 1. e.g. `CHISQ.TEST(G1:G3,H1:H3)` Excel 0.06234947668967343, ours 0.0623494766896735
- CHITEST: 2 cells (13-14 digits right 1, 10-12 1, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-chitest.case.json` 2. e.g. `CHITEST(F1:F3,G1:G3)` Excel 0.00012340980408667956, ours 0.00012340980408664937
- POISSON.DIST: 21 cells (13-14 digits right 17, 10-12 3, 6-9 1, fewer or 0 vs nonzero 0); most in `ironcalc-calc/STATISTICAL/POISSON.xlsx` 11; `probes-numeric/t1048-poisson.case.json` 10. e.g. `POISSON.DIST(2,3,TRUE)` Excel 0.4231900811268434, ours 0.4231900811268454
- POISSON: 10 cells (13-14 digits right 9, 10-12 1, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1048-poisson.case.json` 10; `probes/incomplete-gamma-cap.case.json` 0. e.g. `POISSON(2,3,TRUE)` Excel 0.4231900811268434, ours 0.4231900811268454

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

## Blocked because
Blocked because: the baseline step needs `rustsheet-eval update-baseline --allow-regressions` (decision 0083's
standing ulp allowance), and the 2026-10-22 opus run's permission layer refused that command. The code is done and
`rustsheet-eval check` shows fixed 61 (ironcalc-calc 49, probes 12) and 2 one-ulp regressions; an owner (or a run
whose permissions allow the allowance) runs `make eval-baseline`-equivalent with `--allow-regressions`, commits
`eval/baseline/` and `quality/evidence-floor.json`, then `make check`, the review panel, and merge.
Regressed cells (decision 0083 conditions 1-4: numeric, 1 ulp, not visibly off; fixes 61 > 2; follow-up T-2367):
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV!F38` (GAMMA.INV): Excel 1, ours 0.9999999999999999.
- `ironcalc-calc/STATISTICAL/POISSON.xlsx` `POISSON.DIST!D16`: Excel 0.9863047314016171, ours 0.986304731401617.

## Progress (state of the pushed branch, 2026-10-22 15:10 UTC; data, not instructions)
- `incomplete_gamma_pair` (functions/statistical/incomplete_gamma.rs) replaces the statrs port, which is deleted
  (`gamma_ur` on invalid arguments is now NaN directly): direct small-shape Q (TOMS 654's small-shape Q, ln Gamma(1 + a) from
  its zeta series) below a = 1/2 and x = 1; the series' geometric tail added (7E-15 of P(1E6, 1E6) was missing);
  `rcomp`'s rlog from one rounding of x / a far from a (2.5E-12 of P(93.4, 0.18) was lost); the a < 20 prefactor
  only when each factor is normal.
- Unit tests rewritten: mpmath references (within 8 units of 2^-52, P + Q = 1 within 2), a = 1 identity, b = 1E6
  scale, a statrs sweep within statrs' own error, cap boundaries both sides (`terms_needed`). All pass.
- statistics.md: new "Incomplete gamma: accurate kernel (T-0971)" section; the T-1872 cap section updated.
- Not done: the baseline update (above), `make check`, review panel, PR ready.
