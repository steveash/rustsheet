---
id: T-0971
title: "GAMMA.DIST/CHISQ.DIST/CHISQ.DIST.RT/CHISQ.TEST: accurate regularized incomplete gamma (43 known failures)"
milestone: M9
lane: functions-stat
status: done
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

gen-m9 sweep (gap triage 2026-10-23, T-2255). CHIDIST, CHISQ.DIST, GAMMADIST and GAMMA.DIST values visibly off (68 cells).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 68 known-failure lines in eval/baseline/gen-m9.txt, 68 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A178 `=CHISQ.DIST(CHISQ.DIST.RT(1.5,7)%,8,(PI()=(Data!$B9+1)))`: Excel 9.825189899531443e-09, ours 9.825189899531451e-09
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A178, A231
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A253, A300
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A87
- gen-m9/165db58d1540f014.case.json (1): Calc!A66
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A324, A383
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A119
- gen-m9/3a25a03eb94fcfbd.case.json (3): Calc!A158, A255, A341
- gen-m9/3e98b26cf6713e77.case.json (2): Calc!A268, A329
- gen-m9/40425341c5034eed.case.json (1): Calc!A224
- gen-m9/4250f880ab640c49.case.json (1): Calc!A331
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A224
- gen-m9/4bad303d32cc3f46.case.json (4): Calc!A38, A279, A293, A340
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A97
- gen-m9/530c636749a85209.case.json (2): Calc!A12, A150
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A382
- gen-m9/68d8ffb69ff739d8.case.json (4): Calc!A65, A325, A341, A386
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A270
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A23
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A72
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A319, A333
- gen-m9/8f2ac25945af87da.case.json (2): Calc!A108, A352
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A349
- gen-m9/984cba67bbe6972b.case.json (5): Calc!A42, A91, A140, A175, A252
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A255
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A209, A257
- gen-m9/a54c738c697617c3.case.json (3): Calc!A29, A49, A212
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A57
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A216, A400
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A125
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A388
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A13, A30
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A2, A59
- gen-m9/cde551ea787f713a.case.json (1): Calc!A390
- gen-m9/d8266bc15533702a.case.json (1): Calc!A17
- gen-m9/e6cf3b20a014e6a3.case.json (3): Calc!A162, A223, A345
- gen-m9/ecc87f80077959e0.case.json (3): Calc!A2, A170, A212
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A49
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A104, A249

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: CHIDIST, CHISQ.DIST, GAMMADIST and GAMMA.DIST last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 82 known-failure lines in eval/baseline/gen-m9.txt, 82 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A178 `=CHISQ.DIST(AGGREGATE(5,(Data!A1*7),Data!A2:B2),(Data!D10^STDEV.P({0.02,0.027},Data!A2:A6,Data!D10:D11)),AND(Data!E7:E10,(Data!B9=1.005)))`: Excel 12615662.610100787, ours 12615662.610100795
Cells:
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A178, A367
- gen-m9/146b46e74f2dac3b.case.json (3): Calc!A60, A148, A172
- gen-m9/27601819397dd56e.case.json (1): Calc!A7
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A53
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A51, A86, A295
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A308, A333
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A157, A249
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A97, A133
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A220
- gen-m9/40425341c5034eed.case.json (2): Calc!A132, A316
- gen-m9/4250f880ab640c49.case.json (1): Calc!A125
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A354
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A386
- gen-m9/530c636749a85209.case.json (1): Calc!A79
- gen-m9/60b0b6e69ad35a45.case.json (3): Calc!A138, A209, A251
- gen-m9/623663cacea53bbc.case.json (1): Calc!A282
- gen-m9/7674c89c824aca43.case.json (2): Calc!A102, A332
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A259
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A316
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A84
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A118, A394
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A302
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A150
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A105
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A28, A49
- gen-m9/9cca17438a7e6f8a.case.json (4): Calc!A133, A150, A332, A377
- gen-m9/a013a45a8fbcab4f.case.json (3): Calc!A26, A91, A274
- gen-m9/a54c738c697617c3.case.json (1): Calc!A354
- gen-m9/a612e89321a6df9c.case.json (2): Calc!A181, A227
- gen-m9/a61e547a1cab994c.case.json (3): Calc!A117, A239, A362
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A6, A118, A189
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A152
- gen-m9/b4173fe25109a238.case.json (1): Calc!A366
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A253
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A32, A262
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A217
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A35, A165
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A272
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A290
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A318, A361
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A171, A216
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A86, A330
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A124
- gen-m9/cde551ea787f713a.case.json (2): Calc!A24, A240
- gen-m9/d8266bc15533702a.case.json (2): Calc!A230, A291
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A92
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A23, A205
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A63
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A3
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A1

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 68 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] No line listed above remains NumericClose (`rustsheet-eval show`), at least 25 of the 43 lines
      leave `eval/baseline/ironcalc-calc.txt`, no new lines in any baseline (POISSON.DIST must not
      regress if it switches kernel; EXPON.DIST stays fixed).
- [x] Remaining lines (<= 4 ulp) listed in the PR with expected/actual; they wait for the T-1045
      incomplete-gamma sweep (family 6).
- [x] CHISQ.TEST and CHISQ.DIST.RT use the direct upper tail (no `1.0 - cdf`).
- [x] Unit tests: P(1,x) = -expm1(-x), P(a,x) + Q(a,x) = 1 within 2 ulp at a few points, a in {0.5,
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

## Outcome
`statistical/incomplete_gamma.rs` `incomplete_gamma_pair` replaces statrs' `gamma_lr`/`gamma_ur`, and the statrs port
is deleted. The kernel:
- the prefactor from `powf * exp / tgamma` below a = 20, and TOMS 654's `rcomp` above it;
- the power series with its geometric tail for P below x = a + 1, and modified Lentz for Q above it;
- for a small shape, Q is formed directly. P is 1 - Q wherever that Q is at most 1/2, so
  `GAMMADIST(0.5,1E-300,1,TRUE)` is Excel's 1;
- a = 1 is (-expm1(-x), exp(-x)).

CHISQ.DIST.RT, CHIDIST, CHISQ.TEST and CHITEST take Q directly. The densities of GAMMA.DIST, GAMMADIST and CHISQ.DIST
(`gamma_density`) are the same prefactor at x / scale divided by x, and e^(-x/scale) / scale at a shape of 1.
statistics.md "Incomplete gamma: accurate kernel (T-0971)" records both.

Eval, merged with main at 8c586da (2026-10-25):
- gen-m9 5,664 -> 5,535 known failures (142 fixed);
- ironcalc-calc 737 -> 682 (57 fixed; 29 of the 43 lines listed above);
- probes 2,570 -> 2,558 (12 fixed).

Across every suite, ungated probes-numeric included, in-scope visibly-off cells fell from 11,309 to 10,916.

The baseline was recorded with `update-baseline --allow-regressions` under decision 0083's standing allowance, with
the owner's approval in session https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr. It adds 15 lines. All are
numeric, 1-2 ulp and not visibly off:
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV!F38` (Excel 1, ours 0.9999999999999999);
- `POISSON.xlsx` `POISSON.DIST!D16` (0.9863047314016171 vs 0.986304731401617);
- 13 gen-m9 cells: CHIINV, GAMMA.INV, POISSON, CHIDIST, CHISQ.INV and CHISQ.INV.RT. T-2367 lists them with their values.

The criteria that are not met:
- **Visible parity.** 56 of the 68 gen-m9 cells reached it. The other 12 are still visibly off. So are four ironcalc-calc
  cells, BETA_GAMMA L3, CHISQ G3 and G4, and GAMMA E16 (BETA_GAMMA L4 and GAMMA E21 are still NumericClose too, so the second criterion is not met either, but they
  are now within the 15th digit). Five gen-m9 cells that were already known failures went from within the 15th digit
  on statrs to visibly off: `CHIINV(0.975,5)` twice, `CHISQ.DIST(1E-10,8,...)`, `GAMMADIST(...,7,...)` at 2.9E-64,
  and `CHIDIST(...,8)` at 4.3E-18. Decision 0083's condition 2 does not hold for these five (see the PR). On the
  ones checked against 50 digits, ours is correctly rounded and Excel 7-37 ulp off. So Excel's tail forms need
  fitting. T-2638 (campaign: visible) owns all 21 cells.
- **Bit-exact residue.** The ulp-level cells, 1-4 ulp, are owned by T-2367, along with the cap-shape measurement.
  Ungated probes-numeric has 14 cells newly 1 ulp off.
