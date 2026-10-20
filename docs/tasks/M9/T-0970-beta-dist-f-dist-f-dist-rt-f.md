---
id: T-0970
title: "BETA.DIST/F.DIST/F.DIST.RT/F.TEST: accurate regularized incomplete beta (45 known failures)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1744]
---

## Goal
One accurate regularized incomplete beta I_x(a,b) (and its complement, computed directly) in the
engine, used by BETA.DIST, F.DIST, F.DIST.RT and F.TEST, replacing statrs' `beta_reg`.

## Context
Kernel map (T-1522, decision 0056): rank 3 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric cases `m9-beta-dist-a*`, `m9-f-dist`, `m9-f-dist-rt`, `m9-beta-inv`, `m9-f-inv`, `m9-f-inv-rt` (T-1044).
Triage T-0903. Kernel: `statrs::function::beta::beta_reg` (Numerical-Recipes Lentz continued fraction
whose prefactor uses statrs' Lanczos `ln_gamma`; ~1e-15..1e-13 relative error here), reached through
`statrs::distribution::Beta::cdf`/`pdf` (`functions/statistical/beta.rs` `fn_beta_dist`) and
`FisherSnedecor::cdf`/`pdf` (`functions/statistical/fisher.rs` `fn_f_dist`, `fn_f_dist_rt` and `fn_f_test`,
which compute `1.0 - dist.cdf(x)` instead of the complement). The Beta density goes through statrs'
`gamma(a+b)/(gamma(a)gamma(b))` for a, b <= 80.
45 baseline lines in `eval/baseline/ironcalc-calc.txt` (36 NumericClose, 9 NumericUlp):
- `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` `Sheet1`: G2, G3(C), G4(C), G5(C), G9(C), G11(C), G14(C), G15(C), G16, G19(C), G21, G23(C), G24(C), G25(C), G26(C), G29(C), G30(C), G31(C), G32(C), G42(C), G43(C), G44(C), G45(C), G46(C), G47(C), G48(C), G49(C), G52(C), G60, G61, G63, G64(C), G65(C), G66, G67
- `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` `BETA_DIST`: G3(C), G4(C)
- `ironcalc-calc/STATISTICAL/F_DIST.xlsx` `Sheet1`: F2(C), H2(C), F3(C), H3(C)
- `ironcalc-calc/STATISTICAL/F_TEST.xlsx` `Sheet1`: D1(C), F1, K1(C), F3(C)
(C = NumericClose, unmarked = NumericUlp.)

Examples (expected vs actual):
- `BETA.DIST(2,8,10,FALSE,1,3)` BETA_GAMMA G4: expected 1.4837646484375 (exact), actual 1.4837646484376519 (1e-13).
- `BETA.DIST(2,8,10,TRUE,1,3)` BETA_GAMMA G3: expected 0.6854705810546873, actual 0.685470581054654
  (note Excel is not the exact 0.6854705810546875 = 44923/65536 here: it is Excel's own last digit).
- BETADIST_BETAINV G29: expected 0.6903551301985376, actual 0.6903551301986247 (1.3e-13); G30
  expected 2.3817091316180232e-30, actual 2.3817091316182317e-30.
- `F.DIST(0.7,4,2.4,FALSE)` F3: expected 0.40509259259259256, actual 0.4050925925925941;
  `F.DIST.RT` H3 expected 0.6597222222222223, actual 0.6597222222222209.
- `F.TEST(A1:A5,B1:B5)` D1: expected 0.3266664555668095, actual 0.32666645556680884.

Hypothesis (the oracle is the judge): Excel uses a high-accuracy incomplete beta (e.g. TOMS 708
BRATIO of DiDonato and Morris, or Cephes `incbet` with its power-series / continued-fraction switch),
evaluating the upper tail as I_{1-x}(b,a) rather than 1 - I_x(a,b), with the prefactor
x^a (1-x)^b / (a B(a,b)) from an accurate ln-beta (`special::ln_gamma` is fdlibm `lgamma`). For the
density try `exp((a-1)ln x + (b-1)ln(1-x) - ln B(a,b))` versus the gamma-ratio product and keep the
one that matches. Implement once in `functions/statistical/special.rs`; T-0969 (T.TEST type 3),
T-0974 (BETA.INV/F.INV) and possibly T-0927 reuse it. Reference: `rg beta_reg` should show one
definition afterwards.

## Acceptance criteria
- [ ] No BETA.DIST/F.DIST/F.DIST.RT/F.TEST line listed above remains NumericClose (`rustsheet-eval show`),
      at least 25 of the 45 lines leave `eval/baseline/ironcalc-calc.txt`, no new lines in any
      baseline (BINOM/NEGBINOM must not regress if they switch kernel).
- [ ] Remaining lines (<= 4 ulp) listed in the PR with expected/actual; they wait for the T-1044
      incomplete-beta sweep (family 5).
- [ ] F.DIST.RT and F.TEST use the directly computed upper tail (no `1.0 - cdf`).
- [ ] Unit tests: I_x(a,1) = x^a, I_x(1,b) = 1-(1-x)^b, symmetry I_x(a,b) = 1 - I_{1-x}(b,a) at a few
      points, a or b tiny (1e-4) and large (1e3); bounded iteration counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
BETA.INV/F.INV/F.INV.RT (T-0974); Student t (T-0969); discrete distributions (T-0927); legacy
BETADIST/FDIST/FTEST evidence (T-1047).

## Fit result (T-1744)
Implement Excel's closed forms first: I_x(a,1) = exp(a ln x) (27/27 b = 1 cells; not pow) and I_x(1,b) = 1 - exp(b ln(1-x)) (22/24 a = 1 cells; 1 - pow(1-x,b) fits equally; the misses at x = 0.9, b = 2.5 and 5 are unfitted). The best general candidate, `best_combination` in `fitlab/kernels/beta.py`, is bit-exact on 158 of 283 BETA.DIST cells, against 87 today.
