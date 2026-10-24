---
id: T-2361
title: "Fit Excel's F.TEST kernel: bit-exact two-tailed p (not 2*MIN of its F.DIST)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.TEST/FTEST match Excel to the last digit. At minimum, the visibly-off cells T-1927 carried reach
15-digit parity without losing any cell that is bit-exact today.

## Context
Moved from T-1927 (its visible-parity criterion was not met). Visibly-off cells (gap triage
2026-10-21, decision 0083): probes-numeric/m9-legacy-ftest Calc!A11 `FTEST(B1:C5,B1:B5)` (Excel
0.9873866645220285), probes/stat-argument-modes Sheet1!H249/H250 (0.7947858780819458) and
H251 `FTEST(C1:C6,B1:B8)` (0.9195979047175995). The F tails are `f_cdf`/`f_sf` in
crates/rustsheet-engine/src/functions/statistical/fisher.rs (`fn_f_test`).
Findings so far (docs/excel-semantics/statistics.md "F.TEST / FTEST is not correctly rounded"):
- Excel's F.TEST is not `2*MIN` of its own F.DIST/F.DIST.RT (probes/ftest-decomposition A7/A8,
  B7/B8), and it is exactly symmetric in its two arguments.
- Over the 33 distinct inputs on file, Excel is -6..+4 ulps from the exact p, on both sides. Exact
  p is bit-exact on 12, the A&S 26.6.4-26.6.8 finite series in doubles on 11, the engine today on
  fewer, and none of two-pass/textbook/corrected variance or f = larger/smaller fits all.
  ironcalc-calc STATISTICAL/F_TEST.xlsx F1 (df 3,3) is bit-exact today and the exact p loses it.
- The T-1927 handoff tried an exact half-integer recurrence (1 ulp off on df 7,7 and 9,4).
The F.TEST, F.DIST and BETA.DIST rows of probes/ftest-decomposition in eval/baseline/probes.txt
are this task's (its DEVSQ rows are T-1749's).
Next: use the fitlab (tools, `kernels/beta.py`), with the 33 inputs as a case set. Candidates:
Excel's BETA.DIST kernel run on the reflected side; a series in
x = d2/(d2+d1 f) summed in reverse; long-double (x87) intermediates (decision on "Division rounds
twice"). Then a probe sweep over df 1..10 x a few f to separate the hypotheses.

gen-m9 sweep (gap triage 2026-10-23, T-2255). F.TEST and FTEST values visibly off (66 cells): deep tails where
ours underflows to 0 (`F.TEST(Data!C5:D9,Data!A5:C6)` Excel 3.038000000000038E-29, ours 0) and two-sided p values off
in the 14th-15th digit (`FTEST(Data!B4:E12,Data!B11:E12)` Excel 0.031997240492058855, ours ...925).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 66 known-failure lines in eval/baseline/gen-m9.txt, 66 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A227 `=FTEST(Data!D4:E9,Data!A7:A10)`: Excel 0.5821243127229592, ours 0.5821243127229584
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A227, A356
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A109
- gen-m9/165db58d1540f014.case.json (1): Calc!A245
- gen-m9/27601819397dd56e.case.json (3): Calc!A114, A252, A390
- gen-m9/2801c1c11c21e134.case.json (3): Calc!A168, A246, A351
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A158
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A123, A306
- gen-m9/40425341c5034eed.case.json (1): Calc!A378
- gen-m9/4250f880ab640c49.case.json (1): Calc!A206
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A13
- gen-m9/4bad303d32cc3f46.case.json (3): Calc!A95, A144, A400
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A59
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A390
- gen-m9/58e3253bbd3a7a82.case.json (4): Calc!A141, A147, A285, A371
- gen-m9/7674c89c824aca43.case.json (1): Calc!A315
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A221
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A87
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A398
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A315
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A60
- gen-m9/8f2ac25945af87da.case.json (2): Calc!A119, A241
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A218
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A104, A247
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A167
- gen-m9/a7bf0d8d81d33ba6.case.json (2): Calc!A81, A162
- gen-m9/b6593d7dd13f2dac.case.json (3): Calc!A123, A283, A306
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A27, A88
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A377
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A375
- gen-m9/caa1eed48b1883fe.case.json (3): Calc!A224, A258, A346
- gen-m9/cde551ea787f713a.case.json (3): Calc!A34, A217, A321
- gen-m9/d8266bc15533702a.case.json (2): Calc!A166, A232
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A312, A358
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A63, A155
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A105, A151, A197
- gen-m9/f34f66fb7f81e673.case.json (4): Calc!A4, A46, A92, A326
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A36

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: F.TEST and FTEST last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 119 known-failure lines in eval/baseline/gen-m9.txt, 119 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A349 `=FTEST(Data!B7:E8,Data!B4:E8)`: Excel 1.085009520517214e-17, ours 1.0850095205172149e-17
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A349
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A134, A317
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A13, A59
- gen-m9/165db58d1540f014.case.json (1): Calc!A15
- gen-m9/27601819397dd56e.case.json (3): Calc!A130, A298, A314
- gen-m9/2801c1c11c21e134.case.json (3): Calc!A2, A63, A368
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A219, A280
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A223
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A9, A70
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A191
- gen-m9/3e98b26cf6713e77.case.json (2): Calc!A156, A374
- gen-m9/40425341c5034eed.case.json (1): Calc!A79
- gen-m9/4250f880ab640c49.case.json (2): Calc!A81, A386
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A201
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A205, A266
- gen-m9/4e55500bb8ad3cd8.case.json (2): Calc!A230, A322
- gen-m9/530c636749a85209.case.json (1): Calc!A364
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A252, A393
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A95
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A233, A300
- gen-m9/623663cacea53bbc.case.json (2): Calc!A249, A339
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A21
- gen-m9/7674c89c824aca43.case.json (2): Calc!A177, A193
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A313, A369
- gen-m9/7c20351562a2d71d.case.json (2): Calc!A28, A133
- gen-m9/85f48d5cdfb24555.case.json (3): Calc!A154, A276, A337
- gen-m9/899f9555e67da2d6.case.json (5): Calc!A39, A131, A177, A229, A361
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A182
- gen-m9/8dd2312c96effb1b.case.json (3): Calc!A55, A280, A341
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A375
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A157, A198
- gen-m9/9cca17438a7e6f8a.case.json (3): Calc!A3, A125, A369
- gen-m9/a013a45a8fbcab4f.case.json (5): Calc!A17, A75, A78, A139, A322
- gen-m9/a5828d3925575713.case.json (3): Calc!A148, A222, A286
- gen-m9/a612e89321a6df9c.case.json (2): Calc!A10, A148
- gen-m9/a61e547a1cab994c.case.json (2): Calc!A11, A363
- gen-m9/a7bf0d8d81d33ba6.case.json (2): Calc!A20, A40
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A198, A290
- gen-m9/b4173fe25109a238.case.json (1): Calc!A256
- gen-m9/b6593d7dd13f2dac.case.json (4): Calc!A100, A161, A184, A367
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A327, A347
- gen-m9/bbdfc06cae0e0d71.case.json (3): Calc!A10, A310, A378
- gen-m9/bc40d37cb4d8301d.case.json (3): Calc!A31, A210, A336
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A72
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A141, A385
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A192, A314
- gen-m9/c6cfda6bd10cae93.case.json (3): Calc!A42, A44, A286
- gen-m9/caa1eed48b1883fe.case.json (3): Calc!A136, A319, A380
- gen-m9/cde551ea787f713a.case.json (1): Calc!A199
- gen-m9/d8266bc15533702a.case.json (5): Calc!A110, A288, A293, A349, A354
- gen-m9/e6cf3b20a014e6a3.case.json (4): Calc!A206, A239, A300, A361
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A36
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A201
- gen-m9/f2bd5f380ec002bf.case.json (2): Calc!A243, A335
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A188, A280
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A279
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A166

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 66 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the 4 cells above have Excel's value to 15 significant digits, with no new
  known-failure lines in any baseline (`make eval-baseline` accepts).
- [ ] Either a bit-exact F.TEST kernel (H248/H251 exact), or the fit result written in
  docs/excel-semantics/statistics.md with the best candidate's score.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.
