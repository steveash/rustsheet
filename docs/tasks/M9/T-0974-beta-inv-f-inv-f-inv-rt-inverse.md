---
id: T-0974
title: "BETA.INV/F.INV/F.INV.RT: inverse incomplete beta parity (28 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-0970, T-1044, T-1745]
---

## Goal
BETA.INV, F.INV and F.INV.RT return Excel's values from one inverse-incomplete-beta routine built on
the T-0970 kernel.

## Context
Kernel map (T-1522, decision 0056): rank 5 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric cases `m9-beta-dist-a*`, `m9-f-dist`, `m9-f-dist-rt`, `m9-beta-inv`, `m9-f-inv`, `m9-f-inv-rt` (T-1044).
Triage T-0903. Kernel: `statrs::function::beta::inv_beta_reg` via `Beta::inverse_cdf`
(`functions/statistical/beta.rs` `fn_beta_inv`, then `A + (B-A) t`) and `FisherSnedecor::inverse_cdf`
(`functions/statistical/fisher.rs` `fn_f_inv`, `fn_f_inv_rt` which inverts `1.0 - p`).
28 baseline lines in `eval/baseline/ironcalc-calc.txt` (13 NumericClose, 13 NumericUlp, 2 NumericMismatch):
- `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` `Sheet1`: H3, H4(C), H5(C), H11(M), H15(C), H16(M), H19, H22, H24(C), H25, H26(C), H28(C), H29(C), H31(C), H32, H43(C), H45(C), H60, H61, H63, H64, H65, H72(C)
- `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` `BETA_DIST`: J3
- `ironcalc-calc/STATISTICAL/F_DIST.xlsx` `Sheet1`: J2, L2, J3(C), L3(C)
(C = NumericClose, M = NumericMismatch, unmarked = NumericUlp.)

Examples (expected vs actual):
- H11 `BETA.INV(BETA.DIST(1E-12,2,2.5,TRUE,0,1.2),2,2.5,0,1.2)`: expected 1.0000000000000012e-12, actual
  1.0000000337481705e-12 (Mismatch, 3.4e-8: statrs loses the deep lower tail).
- H16 (x = A = -1): expected -2.220446049250313e-16, actual -1.1102230246251565e-16 (Excel does not land
  on 0; see `docs/excel-semantics/statistics.md`, "Inverse distribution functions do not round-trip exactly").
- H5: expected 33587.000000000226, actual 33586.999999993524 (2e-13); H24 (alpha = 1e-4): expected
  0.23400000000003063, actual 0.23399999999973403.
- `F.INV(F.DIST(0.7,4,2.4,FALSE)...)` J3: expected 0.8753979410454287, actual 0.8753979410454386.

Hypothesis (the oracle is the judge): Excel solves I_t(a,b) = p iteratively on its own BETA.DIST
kernel (T-0970) and maps back with `A + (B-A) t`; F.INV.RT solves on the upper tail directly.
**Needs an oracle sweep (T-0018):** the last digits depend on the solver, so fit against the
T-1044 incomplete-beta sweep, never blind.

gen-m9 sweep (gap triage 2026-10-23, T-2255). F.INV, FINV, F.INV.RT and BETAINV at moderate probabilities (1E-8 < p < 1 - 1E-8,
df below 1E6) where ours is more than half a unit in the 15th digit off (95 cells), plus one GAMMALN.PRECISE whose
argument is such an F.INV.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 96 known-failure lines in eval/baseline/gen-m9.txt, 96 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A355 `=F.INV.RT(CHOOSE(1,Data!B10,(Data!D6/Data!E6),"abc"),6,2.5)`: Excel 2.640308353392362, ours 2.64030835339234
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A355
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A183
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A192
- gen-m9/165db58d1540f014.case.json (3): Calc!A50, A150, A196
- gen-m9/27601819397dd56e.case.json (3): Calc!A10, A81, A148
- gen-m9/2801c1c11c21e134.case.json (3): Calc!A10, A120, A321
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A44, A218
- gen-m9/320d51765af9b6b7.case.json (3): Calc!A15, A199, A379
- gen-m9/35e8168aae2b00f8.case.json (3): Calc!A56, A117, A369
- gen-m9/3a25a03eb94fcfbd.case.json (3): Calc!A105, A151, A334
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A382
- gen-m9/40425341c5034eed.case.json (3): Calc!A108, A246, A367
- gen-m9/4ab2ba98d26c8a5e.case.json (2): Calc!A18, A390
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A56
- gen-m9/4e55500bb8ad3cd8.case.json (2): Calc!A148, A332
- gen-m9/530c636749a85209.case.json (3): Calc!A63, A175, A316
- gen-m9/589f42e67f4b4b8d.case.json (4): Calc!A120, A221, A231, A365
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A272
- gen-m9/60b0b6e69ad35a45.case.json (3): Calc!A150, A197, A258
- gen-m9/623663cacea53bbc.case.json (2): Calc!A70, A340
- gen-m9/7674c89c824aca43.case.json (1): Calc!A212
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A392
- gen-m9/899f9555e67da2d6.case.json (3): Calc!A49, A184, A249
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A283, A357
- gen-m9/8f2ac25945af87da.case.json (2): Calc!A225, A283
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A341
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A37
- gen-m9/a5828d3925575713.case.json (2): Calc!A164, A328
- gen-m9/a61e547a1cab994c.case.json (3): Calc!A155, A268, A297
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A191, A367
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A188
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A269
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A345
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A169, A260
- gen-m9/c18dbf21f11093f3.case.json (4): Calc!A92, A190, A213, A400
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A198, A267
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A359
- gen-m9/d8266bc15533702a.case.json (3): Calc!A39, A272, A394
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A135, A351
- gen-m9/eced4bb6a0234c9e.case.json (6): Calc!A202, A289, A291, A295, A340, A381
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A12, A180
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A29, A56, A75
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A249, A314
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A101
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A244, A336

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: F.INV, F.INV.RT, FINV and BETAINV last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 51 known-failure lines in eval/baseline/gen-m9.txt, 51 formulas.
Examples:
- gen-m9/06b4e8f5de409d5e Calc!A145 `=F.INV.RT(0.05,2.5,3)`: Excel 9.552094495921159, ours 9.552094495921166
Cells:
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A145
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A72
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A5, A51
- gen-m9/27601819397dd56e.case.json (2): Calc!A376, A378
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A241
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A327, A388
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A222
- gen-m9/40425341c5034eed.case.json (3): Calc!A45, A205, A229
- gen-m9/4250f880ab640c49.case.json (1): Calc!A60
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A155
- gen-m9/530c636749a85209.case.json (2): Calc!A270, A339
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A350, A396
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A80
- gen-m9/623663cacea53bbc.case.json (1): Calc!A35
- gen-m9/68d8ffb69ff739d8.case.json (2): Calc!A1, A319
- gen-m9/7674c89c824aca43.case.json (3): Calc!A166, A350, A375
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A221
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A230
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A114
- gen-m9/a54c738c697617c3.case.json (2): Calc!A216, A311
- gen-m9/a5828d3925575713.case.json (1): Calc!A69
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A75
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A272
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A145
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A337
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A101
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A339
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A67
- gen-m9/c18dbf21f11093f3.case.json (2): Calc!A167, A184
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A241
- gen-m9/cde551ea787f713a.case.json (2): Calc!A123, A306
- gen-m9/d8266bc15533702a.case.json (1): Calc!A344
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A204
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A18
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A318
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A19
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A30, A106

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 96 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Both NumericMismatch lines (H11, H16) and all NumericClose lines above leave those classes, at
      least 15 of the 28 lines leave `eval/baseline/ironcalc-calc.txt`, no new lines in any baseline;
      remaining lines listed in the PR with expected/actual.
- [ ] No statrs `inverse_cdf` left for Beta or FisherSnedecor (`rg inverse_cdf`); bounded iteration.
- [ ] `make check` green; reviewers approve.

## Out of scope
The CDF kernel (T-0970); T.INV (T-0967); legacy BETAINV/FINV evidence (T-1047).

## Fit result (T-1745)
Excel's BETA.INV returns a crossing point of its own BETA.DIST at 155 of 188 probe points (statistics.md, "BETA.INV solves Excel's own CDF to the crossing"). Solve the engine's BETA.DIST to adjacent doubles rather than to a loose tolerance. With the correctly rounded I_x that gets 87 of 212 cells against 34 today; 11 of Excel's points miss the crossing by 1 to 188 ulps of p, so the stop rule is still open. The fit's "mid" end is the even-significand end of the final bracket, not a rule to copy. The same solver through the F and t mappings beats the engine on F.INV, F.INV.RT, T.INV and T.INV.2T (fits `f-inv`, `f-inv-rt`, `t-inv`, `t-inv-2t`). Parity improves as T-0970's forward kernel does.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 381 in total.
- BETA.INV: 249 cells (13-14 digits right 208, 10-12 26, 6-9 1, fewer or 0 vs nonzero 14); most in `probes-numeric/t1745-beta-inv-roundtrip.case.json` 109; `probes-numeric/m9-beta-inv.case.json` 102. e.g. `BETA.INV(0.5,1,10)` Excel 0.06696700846319258, ours 0.06696700846319285
- BETAINV: 7 cells (13-14 digits right 6, 10-12 1, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-betainv.case.json` 7; `gen-m9/288b99797f58f149.case.json` 0. e.g. `BETAINV(0.3,10,1.5)` Excel 0.8361662533038416, ours 0.8361662533038301
- F.INV: 43 cells (13-14 digits right 29, 10-12 5, 6-9 0, fewer or 0 vs nonzero 9); most in `probes-numeric/m9-f-inv.case.json` 20; `probes-numeric/t1049-finv.case.json` 13. e.g. `F.INV(0.4,5,5)` Excel 0.7878805312664828, ours 0.7878805312664846
- F.INV.RT: 57 cells (13-14 digits right 37, 10-12 7, 6-9 8, fewer or 0 vs nonzero 5); most in `probes-numeric/m9-legacy-finv.case.json` 21; `probes-numeric/m9-f-inv-rt.case.json` 19. e.g. `F.INV.RT(0.5,2,5)` Excel 0.7987697769322353, ours 0.7987697769322422
- FINV: 25 cells (13-14 digits right 14, 10-12 4, 6-9 4, fewer or 0 vs nonzero 3); most in `probes-numeric/m9-legacy-finv.case.json` 23; `probes-numeric/t1530-f-df-cut.case.json` 2. e.g. `FINV(0.5,2,5)` Excel 0.7987697769322353, ours 0.7987697769322422

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.
