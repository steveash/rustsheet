---
id: T-0970
title: "BETA.DIST/F.DIST/F.DIST.RT/F.TEST: accurate regularized incomplete beta (45 known failures)"
milestone: M9
lane: functions-stat
status: done
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
- [ ] (Partly: deferred to T-2119; BETADIST_BETAINV G5 and G26 remain NumericClose) No BETA.DIST/F.DIST/F.DIST.RT/F.TEST line listed above remains NumericClose (`rustsheet-eval show`),
      at least 25 of the 45 lines leave `eval/baseline/ironcalc-calc.txt`, no new lines in any
      baseline (BINOM/NEGBINOM must not regress if they switch kernel).
- [x] Remaining lines (<= 4 ulp) listed in the PR with expected/actual; they wait for the T-1044
      incomplete-beta sweep (family 5).
- [x] F.DIST.RT and F.TEST use the directly computed upper tail (no `1.0 - cdf`).
- [x] Unit tests: I_x(a,1) = x^a, I_x(1,b) = 1-(1-x)^b, symmetry I_x(a,b) = 1 - I_{1-x}(b,a) at a few
      points, a or b tiny (1e-4) and large (1e3); bounded iteration counts.
- [x] `make check` green; reviewers approve.

## Out of scope
BETA.INV/F.INV/F.INV.RT (T-0974); Student t (T-0969); discrete distributions (T-0927); legacy
BETADIST/FDIST/FTEST evidence (T-1047).

## Fit result (T-1744)
Implement Excel's closed forms first: I_x(a,1) = exp(a ln x) (27/27 b = 1 cells; not pow) and I_x(1,b) = 1 - exp(b ln(1-x)) (22/24 a = 1 cells; 1 - pow(1-x,b) fits equally; the misses at x = 0.9, b = 2.5 and 5 are unfitted). The best general candidate, `best_combination` in `fitlab/kernels/beta.py`, is bit-exact on 158 of 283 BETA.DIST cells, against 87 today.

## Outcome
New `functions/statistical/incomplete_beta.rs`: `beta_reg` is the T-1744 best combination in
Rust. That is Excel's closed forms, Cephes' two continued fractions (capped at 300 steps), the
mean switch and an exp(a ln x)*exp(b ln(1-x))*Gamma ratio prefactor (musl `tgamma`). Past
MAXGAM it uses a TOMS 708 `brcomp`-style prefactor (rlog1 exponent with Stirling corrections, or
`algdiv`), which takes `BETA.DIST(0.555,1000,1000,...)` from 785 to 3 ulp. `beta_density` is the
pow*Gamma-ratio density.
Wired in:
- F.DIST cdf and F.DIST.RT: the upper tail I_w(d2/2,d1/2), computed directly.
- F.TEST: both tails.
- F.DIST pdf: the beta density times y(1-y)/x where statrs' direct form is finite.
- BETA.DIST pdf.

The incomplete beta takes the exact complement y alongside x (TOMS 708 `bratio`). F.DIST.RT
takes the other tail of F.DIST's evaluation (`beta_reg_upper`), except in the closed forms.

Eval (gated): ironcalc-calc known failures 1056 -> 1049 (BETADIST_BETAINV G3, BETA_GAMMA G4,
F_DIST F2, H2, H3, F_TEST F1, F3) and probes 1669 -> 1666 (stat-argument-modes H215, H221,
H226), with no regressions. F_DIST F3 goes from 28 ulp to 1 ulp, F_TEST D1 from 13 to 1 and K1
from 16 to 3.

probes-numeric (gate = false), against a main build: 72 cells gain bit-exact and 3 lose it, a net
gain of 69. The two tail rules behind this are PARTIAL: t1049-fdistrt A2 and A8 contradict them,
see statistics.md.

| case | main | branch |
|---|---|---|
| m9-f-dist | 1 | 10 |
| m9-f-dist-rt | 0 | 8 |
| m9-legacy-fdist | 29 | 42 |
| m9-legacy-ftest | 8 | 12 |
| t1049-ftest | 6 | 14 |
| m9-beta-dist-a0p5 ... a100 (7 cases) | 49 | 73 |

Three cells statrs had exact are 1 ulp off: m9-legacy-fdist A61, A62 (`FDIST(100,1,1)`) and
t1049-fdist A23. They are listed in statistics.md.

Not met here, and why:
- At least 25 of the 45 lines leaving the baseline, and no NumericClose: only 7 left.
- `rg beta_reg` showing one definition: BETA.DIST's cdf still uses statrs.

Measured on this branch, moving BETA.DIST's cdf to `beta_reg` fixes 16 more cells and leaves
8 exact and 23 within 4 ulp of the 33 cumulative cells. But it moves nine chained
BETA.INV(BETA.DIST()) cells from exact to 1-2 ulp, with any inversion tried.
Recording those needs `--allow-regressions` under a decision record. This run was not permitted
to do that, so the switch is T-2119. G5 (160 ulp, Excel correctly rounded) and G26 (30 ulp,
Excel 34 from correct) would stay NumericClose even then. Details are in
docs/excel-semantics/statistics.md "Incomplete beta in the ironcalc-calc workbooks".
Release microbenchmark, 200k calls each: `beta_reg` against statrs `beta_reg`.

| (x, a, b) | `beta_reg` | statrs |
|---|---|---|
| (0.3, 2, 3) | 57 ns | 117 ns |
| (0.5, 1E6, 1E6) | 1.75 us | 2.9 us |
| (0.4999, 1E3, 1E3) | 0.49 us | 1.33 us |
| (0.9, 0.1, 10) | 192 ns | 168 ns |
| (0.5, 5E9, 5E9), F at df 1E10 | 1.95 us | - |

The work is constant per call, and the worst case measured stays under BINOM.DIST's 3.2 us,
so the lift weights are unchanged.

