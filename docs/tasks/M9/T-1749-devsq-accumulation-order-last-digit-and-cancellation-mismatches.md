---
id: T-1749
title: "DEVSQ accumulation order: last-digit and cancellation mismatches (probes/stat-order-floor-batch)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
DEVSQ matches Excel's value to the last digit, including under cancellation.

## Context
Oracle: `DEVSQ(L1:L6)` = 48.833333333333336 (ours ...34), `DEVSQ(0.1,0.2,0.3)` = 0.019999999999999997 (ours ...976), `DEVSQ(1E15,1E15+1)` = 0.5 (ours 0), plus 1-ulp cases A278, A280, A290, A300 in probes/stat-order-floor-batch. More last-digit cells (T-1927): probes/ftest-decomposition Calc!B9 `DEVSQ(Data!C1:C3)` = 4.666666666666666 (ours ...668), C9/F10 77.60000000000001 (ours 77.6), C10/F9 30.8 (ours 30.799999999999997), D10 11.719999999999999 (ours 11.720000000000013). Excel looks like a mean-then-squared-deviation two-pass; fit the accumulation order against these cells (see coercion.md note on I392).

Also poi/FormulaEvalTestData_Copy.xlsx EverythingTests!I392 `DEVSQ("1","2",2)`: Excel 0.6666666666666666, ours 0.6666666666666661 (sumsq - sum^2/n), now visibly off
(half a unit in the 15th digit is 5E-16; owner moved from the deferred T-0951 at the 2026-10-28 triage). A two-pass mean/deviation sum gives Excel's value here; T-0951 keeps the VARP cells.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 1 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on DEVSQ 1. Every one is listed above.

gen-m9 sweep (gap triage 2026-10-23, T-2255). DEVSQ and KURT: `DEVSQ({100000000.1,100000000.2,100000000.3})` Excel 0.020000000596046563,
ours -4 (the one-pass cancellation), and three KURT cells off in the 15th digit.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas.
Examples:
- gen-m9/27601819397dd56e Calc!A200 `=KURT(Data!E4:E11)`: Excel 3.9999999999979536, ours 3.9999999999979607
Cells:
- gen-m9/27601819397dd56e.case.json (1): Calc!A200
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A279
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A334
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A55

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: DEVSQ (124) and KURT (45) last digits (the accumulation order). Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 169 known-failure lines in eval/baseline/gen-m9.txt, 169 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A185 `=KURT(Data!C7:C12,Data!A11:A12,Data!E11:E12)`: Excel 7.000000000000002, ours 7.0
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (3): Calc!A185, A246, A345
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A14, A158
- gen-m9/0a614cfe974b59c3.case.json (5): Calc!A89, A139, A211, A322, A333
- gen-m9/0a9226016e8f5f33.case.json (4): Calc!A74, A88, A166, A212
- gen-m9/146b46e74f2dac3b.case.json (4): Calc!A12, A73, A195, A378
- gen-m9/27601819397dd56e.case.json (2): Calc!A108, A288
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A21, A82, A181
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A32, A246
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A74, A185
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A111, A233
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A113
- gen-m9/3e98b26cf6713e77.case.json (2): Calc!A116, A359
- gen-m9/40425341c5034eed.case.json (3): Calc!A47, A277, A313
- gen-m9/4250f880ab640c49.case.json (3): Calc!A29, A45, A90
- gen-m9/4ab2ba98d26c8a5e.case.json (3): Calc!A137, A183, A356
- gen-m9/4bad303d32cc3f46.case.json (3): Calc!A57, A179, A362
- gen-m9/4e55500bb8ad3cd8.case.json (2): Calc!A44, A182
- gen-m9/530c636749a85209.case.json (2): Calc!A235, A373
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A141, A371
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A50, A280, A314
- gen-m9/623663cacea53bbc.case.json (2): Calc!A163, A346
- gen-m9/68d8ffb69ff739d8.case.json (4): Calc!A25, A28, A89, A330
- gen-m9/7674c89c824aca43.case.json (3): Calc!A13, A151, A335
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A35, A173
- gen-m9/7c20351562a2d71d.case.json (8): Calc!A26, A72, A118, A134, A210, A226, A256, A272
- gen-m9/85f48d5cdfb24555.case.json (4): Calc!A156, A217, A278, A339
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A17, A109
- gen-m9/8d3588d8df0812df.case.json (4): Calc!A91, A152, A335, A396
- gen-m9/8dd2312c96effb1b.case.json (4): Calc!A48, A231, A277, A292
- gen-m9/8f2ac25945af87da.case.json (3): Calc!A28, A211, A333
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A376
- gen-m9/9cca17438a7e6f8a.case.json (3): Calc!A139, A195, A261
- gen-m9/a013a45a8fbcab4f.case.json (3): Calc!A2, A133, A316
- gen-m9/a54c738c697617c3.case.json (3): Calc!A41, A148, A270
- gen-m9/a5828d3925575713.case.json (3): Calc!A133, A225, A271
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A316
- gen-m9/a61e547a1cab994c.case.json (3): Calc!A7, A75, A190
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A140, A200, A383
- gen-m9/a7bf0d8d81d33ba6.case.json (3): Calc!A191, A252, A374
- gen-m9/abc2939ebd1e2365.case.json (4): Calc!A34, A104, A150, A196
- gen-m9/b6593d7dd13f2dac.case.json (2): Calc!A14, A363
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A191, A375
- gen-m9/bbdfc06cae0e0d71.case.json (4): Calc!A114, A123, A160, A353
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A308
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A52
- gen-m9/c18dbf21f11093f3.case.json (3): Calc!A77, A215, A307
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A38, A282
- gen-m9/c468f5046aadfbbc.case.json (4): Calc!A46, A168, A267, A290
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A24, A109
- gen-m9/caa1eed48b1883fe.case.json (3): Calc!A9, A253, A284
- gen-m9/cde551ea787f713a.case.json (2): Calc!A353, A365
- gen-m9/d8266bc15533702a.case.json (3): Calc!A9, A46, A131
- gen-m9/e6cf3b20a014e6a3.case.json (4): Calc!A119, A180, A241, A377
- gen-m9/eced4bb6a0234c9e.case.json (3): Calc!A176, A222, A360
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A309, A350
- gen-m9/f2bd5f380ec002bf.case.json (4): Calc!A85, A223, A269, A361
- gen-m9/f34f66fb7f81e673.case.json (3): Calc!A330, A345, A376
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A49, A293
- gen-m9/fec39efaba9284c3.case.json (6): Calc!A9, A38, A55, A147, A193, A222

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 line in eval/baseline/probes.txt disappears (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: poi/FormulaEvalTestData_Copy.xlsx EverythingTests!I392 is within half a unit of the 15th significant digit of 0.6666666666666666 (its eval/baseline/poi.txt line may stay only if the last digit is still off; no new known-failure lines).
- [ ] A277, A278, A280, A290, A294, A295, A300 pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMSQ and other functions (probe `SUMSQ(1,)` before changing it).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (DEVSQ(1E15,1E15+1) is 0, Excel 0.5), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).
