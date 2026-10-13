---
id: T-0971
title: "GAMMA.DIST/CHISQ.DIST/CHISQ.DIST.RT/CHISQ.TEST: accurate regularized incomplete gamma (43 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One accurate regularized incomplete gamma P(a,x) / Q(a,x) pair (Q computed directly, not as 1 - P)
in the engine, used by GAMMA.DIST, CHISQ.DIST, CHISQ.DIST.RT and CHISQ.TEST, replacing statrs'
`gamma_lr`/`gamma_ur`; the gamma density from the same building blocks.

## Context
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
