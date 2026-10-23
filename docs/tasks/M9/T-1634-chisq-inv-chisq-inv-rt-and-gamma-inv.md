---
id: T-1634
title: "CHISQ.INV, CHISQ.INV.RT and GAMMA.INV tails: match Excel at tiny and near-1 probability (T-1543 probe)"
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
CHISQ.INV, CHISQ.INV.RT and GAMMA.INV match Excel's oracle values for p from 1E-300 to 1-1E-16 (no `#NUM!` where Excel has a value, no wrong digits in the tails).

## Context
probes-numeric/t1543-inverse-cdf-tiny-p (oracle-filled, T-1543) fails at Calc!A25-A28, A29-A33, A35-A36, A41-A46, A49-A61, A65-A66 (run `rustsheet-eval show probes-numeric/t1543-inverse-cdf-tiny-p.case.json`). `statrs`' gamma `inverse_cdf` is wrong in the upper tail (CHISQ.INV(1-1E-16,50) = 179.80, Excel 180.68) and fails in the lower tail; the fix is likely bisection on the exact cdf as in beta.rs `beta_quantile`/special.rs `bisect_unit_interval`. BETA.INV cells (Excel quirks, not implemented): A67-A72, A79 (6.988271195E-61 vs ours 6.988271188E-61), A86, A88, A91-A92; add BETA.INV(0.3,1E6,1E6) and BETA.INV(0.5,1E7,2) oracle cells (shapes 1E5 to 1E8 now bisect). Details in docs/excel-semantics/statistics.md, T-1543 entry.

gen-m9 sweep (gap triage 2026-10-23, T-2255). CHISQ.INV, CHIINV, CHISQ.INV.RT, GAMMA.INV and GAMMAINV in the
tails and at small shapes: ours `#NUM!` or a value off at p < 1E-8, p > 1 - 1E-8 (`CHISQ.INV(1-1E-16,9)` Excel
95.79817039503867, ours 95.009765625; `CHISQ.INV.RT(1-1E-16,4)` Excel 2.9802322535725054E-08, ours 0.0002438445807038453)
and `#NUM!` at moderate p with a shape below 1 or a tiny scale (`CHISQ.INV(0.025,1)` Excel 0.0009820691171752564,
`GAMMAINV(0.5,0.01,100)` 4.4655350189103716E-29): 136 cells.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 136 known-failure lines in eval/baseline/gen-m9.txt, 136 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A132 `=CHISQ.INV(1-1E-10,10)`: Excel 68.16761795190413, ours 68.16761840669743
- gen-m9/06b4e8f5de409d5e Calc!A215 `=GAMMA.INV(AVERAGEIF(Data!B9:E11,"<5",Data!C7:D9),99999,1E-10)`: Excel 1.0004040796843747e-05, ours #NUM!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A132, A315
- gen-m9/06b4e8f5de409d5e.case.json (5): Calc!A128, A194, A215, A250, A311
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A32
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A240, A399
- gen-m9/146b46e74f2dac3b.case.json (3): Calc!A37, A45, A396
- gen-m9/165db58d1540f014.case.json (2): Calc!A132, A283
- gen-m9/27601819397dd56e.case.json (4): Calc!A231, A310, A323, A356
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A100
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A8, A252, A254
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A132
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A23
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A39, A256
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A246, A254
- gen-m9/3e98b26cf6713e77.case.json (3): Calc!A21, A142, A326
- gen-m9/40425341c5034eed.case.json (3): Calc!A75, A121, A305
- gen-m9/4250f880ab640c49.case.json (2): Calc!A5, A127
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A315
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A227
- gen-m9/4e55500bb8ad3cd8.case.json (5): Calc!A61, A79, A111, A217, A249
- gen-m9/530c636749a85209.case.json (3): Calc!A4, A90, A372
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A83, A287, A376
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A88
- gen-m9/623663cacea53bbc.case.json (2): Calc!A98, A184
- gen-m9/68d8ffb69ff739d8.case.json (2): Calc!A5, A52
- gen-m9/7674c89c824aca43.case.json (2): Calc!A198, A290
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A101, A235
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A340
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A125
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A51, A327
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A130
- gen-m9/8dd2312c96effb1b.case.json (3): Calc!A171, A368, A374
- gen-m9/8f2ac25945af87da.case.json (4): Calc!A71, A301, A315, A391
- gen-m9/984cba67bbe6972b.case.json (5): Calc!A13, A146, A162, A232, A246
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A4, A370
- gen-m9/a54c738c697617c3.case.json (1): Calc!A104
- gen-m9/a5828d3925575713.case.json (2): Calc!A338, A372
- gen-m9/a612e89321a6df9c.case.json (3): Calc!A169, A173, A317
- gen-m9/a61e547a1cab994c.case.json (2): Calc!A184, A193
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A293, A348
- gen-m9/a7bf0d8d81d33ba6.case.json (3): Calc!A216, A269, A399
- gen-m9/abc2939ebd1e2365.case.json (3): Calc!A107, A337, A348
- gen-m9/b4173fe25109a238.case.json (2): Calc!A38, A252
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A265
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A268
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A81, A356
- gen-m9/c18dbf21f11093f3.case.json (4): Calc!A107, A153, A291, A301
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A172, A355
- gen-m9/c468f5046aadfbbc.case.json (3): Calc!A14, A187, A315
- gen-m9/caa1eed48b1883fe.case.json (3): Calc!A151, A261, A316
- gen-m9/cde551ea787f713a.case.json (4): Calc!A45, A270, A317, A347
- gen-m9/d8266bc15533702a.case.json (1): Calc!A212
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A269, A281
- gen-m9/ecc87f80077959e0.case.json (3): Calc!A77, A119, A301
- gen-m9/eced4bb6a0234c9e.case.json (4): Calc!A123, A150, A173, A265
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A136, A327
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A100
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A56
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A243, A304
- gen-m9/fec39efaba9284c3.case.json (4): Calc!A107, A144, A236, A374

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): CHIINV, CHISQ.INV and GAMMA.INV tails at tiny p nested in other calls (`STANDARDIZE(7,CHIINV(1e-15,8),1E+5)`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 visibly off.
Examples:
- gen-m9/0620505280d2ea6e Calc!A292 `=STANDARDIZE(7,CHIINV(1e-15,8),1E+5)`: Excel -0.0008136100835821856, ours -0.000812715781421368
- gen-m9/17c110a53924beb8 Calc!A228 `=TRIMMEAN({5},(CHIINV(1E-20,7)*(Data!B5*Data!$C10)))`: Excel 5.0, ours #NUM!
- gen-m9/31f09e6c48ca245c Calc!A191 `=STDEVPA(Data!C5,(Data!B8/GAMMA.INV(1e-15,2.5,100)))`: Excel 3.0922108671216676e-12, ours 9.444314973097486e-13
- gen-m9/a63d1c7d6f96c56d Calc!A145 `=MAXA(CHISQ.INV.RT(1E-10,8),Data!D$1,Data!C7:C8)`: Excel 63.3979644111356, ours 63.397964021600195
- gen-m9/a63d1c7d6f96c56d Calc!A291 `=POISSON.DIST(0,CHISQ.INV(1E-16,COUNTIF(Data!C8:E11,"<>")),TRUE)`: Excel 0.9986596927837208, ours 0.9981518053624893
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A292
- gen-m9/17c110a53924beb8.case.json (1): Calc!A228
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A191
- gen-m9/a63d1c7d6f96c56d.case.json (2): Calc!A145, A291

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `GESTEP(CHISQ.INV(1E-300,2))` is 1 in Excel, so CHISQ.INV of 1E-300 is a number (ours `#NUM!`; gen-m9/e85daedce5c2675e A393).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/e85daedce5c2675e Calc!A393 `=GESTEP(CHISQ.INV(1E-300,2))`: Excel 1.0, ours #NUM!
Cells:
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A393

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 141 known-failure lines (136 from T-2255, 5 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The listed cells of t1543-inverse-cdf-tiny-p pass (within the suite's numeric tolerance) and ordinary-p cases do not regress.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.
