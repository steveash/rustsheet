---
id: T-1742
title: "LOGEST: match Excel on the logest-modes probe (spill shape, flags, multi-column x, ULP-level numerics)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Every cell of probes/logest-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: LOGEST's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/logest-modes (excel-graph values). Mismatches against the engine today: 49 TypeMismatch, 63 NumericClose, 20 NumericUlp, 1 NumericMismatch, 1 ErrorMismatch. Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 8 cells, 1 numeric past half a unit in the 15th digit and 7 with a wrong type, error or text, blamed on LOGEST 8.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/logest-modes Sheet1!K141 | `=LOGEST(D2:D9,A2:A9,"x")` | #VALUE! | 1.7574908101856999 |
| probes/logest-modes Sheet1!K162 | `=LOGEST(B15,A15,TRUE,TRUE)` | 1 | #VALUE! |
| probes/logest-modes Sheet1!K169 | `=LOGEST({1,2,4,8},{1,2,3,4},TRUE,TRUE)` | 1.9999999999999998 | #REF! |
| probes/logest-modes Sheet1!K183 | `=LOGEST({1,2,4,8},{1;2;3;4})` | #REF! | 1.9999999999999993 |
| probes/logest-modes Sheet1!K190 | `=LOGEST({2,4},{1,2})` | 2 | #REF! |
| probes/logest-modes Sheet1!K197 | `=LOGEST({0.5,0.25,0.125},{1,2,3},TRUE,TRUE)` | 0.5 | #REF! |
| probes/logest-modes Sheet1!K71 | `=LOGEST(D2:D9,,TRUE,TRUE)` | 1.7574908101857 | #REF! |
| probes/logest-modes Sheet1!K85 | `=LOGEST(D2:D9,A2:C9,TRUE,TRUE)` | 0.982454941667086 | 0.9824549416670796 |

gen-m9 sweep (gap triage 2026-10-23, T-2255). LOGEST in 167 cells: a scalar known_y is one point (anchor 1; 33 cells), a
non-positive known_y is `#NUM!` before any text or boolean in it is `#VALUE!` (46 cells are `#NUM!`
in Excel, most of them this), the `#REF!` size checks as for
LINEST, and error arguments in Excel's order. statistics.md "LINEST, LOGEST and GROWTH in the gen-m9 sweep (T-2255)".
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 167 known-failure lines in eval/baseline/gen-m9.txt, 167 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A100 `=BINOM.DIST(0,10,1E-10,(LOGEST(Data!A2:D2,Data!A4:D5)<=STDEV.P((-2.5),Data!B6)))`: Excel #VALUE!, ours #REF!
- gen-m9/05d44f6dd6cfc567 Calc!A152 `=LOGEST(Data!C8:E8)`: Excel #VALUE!, ours #NUM!
- gen-m9/05d44f6dd6cfc567 Calc!A213 `=LOGEST(Data!B11:B11,Data!A5:C6,((Data!A11-Data!E8)<Data!$B2),"FALSE")`: Excel #REF!, ours #VALUE!
- gen-m9/06b4e8f5de409d5e Calc!A290 `=LOGEST(123456789.123)`: Excel 1.0, ours #VALUE!
- gen-m9/0a614cfe974b59c3 Calc!A281 `=LOGEST(Data!A11:D12,(_xlfn.PIVOTBY(Data!C1:D12,Data!C1:E12,Data!C1:D12,LAMBDA(acc,acc),1,-1,-1,-1)-AVERAGEIFS(Data!C8:E9,Data!A4:C9,"a*",Data!B10:C11,"abc")),SORTBY(Data!B1:E12,Data!E1:E12,1,Data!E1:E12,1))`: Excel #SPILL!, ours #VALUE!
- gen-m9/0a614cfe974b59c3 Calc!A37 `=LOGEST((-2.5),AVERAGE(-Data!E7))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (3): Calc!A100, A152, A213
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A107, A290
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A37, A281
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A132, A224
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A22
- gen-m9/165db58d1540f014.case.json (5): Calc!A143, A189, A235, A281, A327
- gen-m9/27601819397dd56e.case.json (3): Calc!A94, A232, A324
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A108, A230
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A49, A56, A293
- gen-m9/2a18b2f5a4dc43ac.case.json (5): Calc!A33, A155, A216, A277, A399
- gen-m9/320d51765af9b6b7.case.json (5): Calc!A78, A124, A170, A262, A354
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A217, A339
- gen-m9/3a25a03eb94fcfbd.case.json (3): Calc!A26, A148, A270
- gen-m9/3e98b26cf6713e77.case.json (3): Calc!A32, A93, A398
- gen-m9/40425341c5034eed.case.json (4): Calc!A39, A131, A177, A269
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A118
- gen-m9/4bad303d32cc3f46.case.json (4): Calc!A13, A74, A257, A379
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A336
- gen-m9/530c636749a85209.case.json (3): Calc!A53, A237, A283
- gen-m9/589f42e67f4b4b8d.case.json (5): Calc!A55, A193, A239, A285, A331
- gen-m9/58e3253bbd3a7a82.case.json (4): Calc!A20, A158, A204, A250
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A11
- gen-m9/623663cacea53bbc.case.json (1): Calc!A83
- gen-m9/68d8ffb69ff739d8.case.json (4): Calc!A85, A146, A244, A268
- gen-m9/7674c89c824aca43.case.json (4): Calc!A46, A184, A276, A368
- gen-m9/7bd219ec0c0c5e22.case.json (3): Calc!A190, A236, A282
- gen-m9/7c20351562a2d71d.case.json (3): Calc!A236, A282, A328
- gen-m9/85f48d5cdfb24555.case.json (4): Calc!A181, A324, A332, A364
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A83, A175
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A26, A148
- gen-m9/8dd2312c96effb1b.case.json (3): Calc!A24, A207, A329
- gen-m9/8f2ac25945af87da.case.json (2): Calc!A38, A343
- gen-m9/8fec7b207f4dd753.case.json (5): Calc!A73, A134, A256, A317, A378
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A334
- gen-m9/a54c738c697617c3.case.json (1): Calc!A182
- gen-m9/a5828d3925575713.case.json (3): Calc!A29, A75, A305
- gen-m9/a612e89321a6df9c.case.json (2): Calc!A100, A330
- gen-m9/a61e547a1cab994c.case.json (2): Calc!A162, A284
- gen-m9/a7bf0d8d81d33ba6.case.json (2): Calc!A143, A204
- gen-m9/abc2939ebd1e2365.case.json (4): Calc!A111, A203, A295, A387
- gen-m9/b6593d7dd13f2dac.case.json (3): Calc!A134, A195, A378
- gen-m9/b8ee55934c5b824c.case.json (4): Calc!A161, A207, A299, A345
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A2, A186
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A60, A182
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A94, A338
- gen-m9/c18dbf21f11093f3.case.json (4): Calc!A97, A143, A189, A281
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A242, A285
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A160, A282
- gen-m9/c6cfda6bd10cae93.case.json (3): Calc!A40, A101, A345
- gen-m9/caa1eed48b1883fe.case.json (5): Calc!A12, A73, A134, A256, A317
- gen-m9/cde551ea787f713a.case.json (3): Calc!A19, A80, A202
- gen-m9/d8266bc15533702a.case.json (2): Calc!A56, A239
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A54
- gen-m9/eced4bb6a0234c9e.case.json (4): Calc!A25, A117, A347, A393
- gen-m9/ee2497a5e683c5d9.case.json (3): Calc!A113, A251, A389
- gen-m9/f2bd5f380ec002bf.case.json (4): Calc!A45, A91, A183, A229
- gen-m9/f34f66fb7f81e673.case.json (6): Calc!A41, A87, A133, A225, A271, A317
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A323, A384
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A86

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 167 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the 8 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 8 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/logest-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong LOGEST shapes and errors, and K85 past the 15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The last-digit numerics stay with T-2006.
