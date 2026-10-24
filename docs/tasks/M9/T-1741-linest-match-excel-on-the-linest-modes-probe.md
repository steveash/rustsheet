---
id: T-1741
title: "LINEST: match Excel on the linest-modes probe (spill shape, #N/A fill, collinear x, flags, 1e-16 residual stats)"
milestone: M9
lane: functions-stat
status: blocked
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Every cell of probes/linest-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: LINEST's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/linest-modes (excel-graph values). Mismatches against the engine today: 102 TypeMismatch (stats block cells empty or #N/A where Excel gives numbers, e.g. LINEST(B2:B9,A2:E9,TRUE,TRUE) collinear x gives ~1e-16 values, we give #NUM!), 38 NumericUlp, 32 NumericClose, 11 NumericMismatch, 2 ErrorMismatch (LINEST(B2:B9,A2:A9&"") is #VALUE!). Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 13 cells, 1 numeric past half a unit in the 15th digit and 12 with a wrong type, error or text, blamed on LINEST 13, TRANSPOSE 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/linest-modes Sheet1!K106 | `=LINEST(B2:B9,A2:E9,TRUE,TRUE)` | 1.4342828472961902e-16 | #NUM! |
| probes/linest-modes Sheet1!K113 | `=LINEST(B2:B9,A2:A9&"")` | #VALUE! | #NUM! |
| probes/linest-modes Sheet1!K134 | `=LINEST(A12:A13,B12:B13,TRUE,TRUE)` | 0.5 | 0.49999999999999867 |
| probes/linest-modes Sheet1!K141 | `=LINEST(B15,A15,TRUE,TRUE)` | 0 | #VALUE! |
| probes/linest-modes Sheet1!K155 | `=LINEST(B2:B9,A2:A9,"x")` | #VALUE! | 1.9976190476190487 |
| probes/linest-modes Sheet1!K162 | `=LINEST(B2:B9,A2:A9,TRUE,"x")` | #VALUE! | 1.9976190476190487 |
| probes/linest-modes Sheet1!K169 | `=LINEST(TRANSPOSE(B2:B9),TRANSPOSE(A2:A9),TRUE,TRUE)` | 1.9976190476190474 | #REF! |
| probes/linest-modes Sheet1!K204 | `=LINEST(5,A2:A9)` | 0 | #VALUE! |
| probes/linest-modes Sheet1!K218 | `=LINEST({1,2,3,5},{1,2,3,4},TRUE,TRUE)` | 1.3 | #REF! |
| probes/linest-modes Sheet1!K232 | `=LINEST({1,2,3,5},{1;2;3;4})` | #REF! | 1.2999999999999998 |
| probes/linest-modes Sheet1!K239 | `=LINEST({2,4},{1,2})` | 2 | #REF! |
| probes/linest-modes Sheet1!K246 | `=LINEST({2,4,6},{1,2,3},TRUE,TRUE)` | 2 | #REF! |
| probes/linest-modes Sheet1!K71 | `=LINEST(B2:B9,,TRUE,TRUE)` | 1.9976190476190474 | #REF! |

gen-m9 sweep (gap triage 2026-10-23, T-2255). LINEST in 159 cells (8 of them spilled B cells whose anchor is `#VALUE!` in Excel):
a scalar known_y with no known_x is one point (anchor 0; 42 cells), a scalar known_y beside a multi-cell known_x, or a
range known_y beside a one-cell text known_x, is `#REF!` (size before text), error arguments in Excel's order, and a range
known_y holding text is `#VALUE!` with nothing spilled. statistics.md "LINEST, LOGEST and GROWTH in the gen-m9 sweep
(T-2255)".
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 159 known-failure lines in eval/baseline/gen-m9.txt, 151 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A244 `=LOGNORM.INV((LINEST(2.5,Data!B6:D12)-CHISQ.TEST(Data!A10:A11,Data!A8:C10)),ROW(),SUMIFS(Data!B4:D6,Data!C1:E7,"<>"))`: Excel #REF!, ours #VALUE!
- gen-m9/05d44f6dd6cfc567 Calc!A378 `=LINEST(2.5)`: Excel 0.0, ours #VALUE!
- gen-m9/06b4e8f5de409d5e Calc!A121 `=LINEST(SUMPRODUCT(Data!C10:D12,0.5),GAMMADIST(VARA(0.2,Data!D8:D10,Data!E10:E10),0.999,((-2.5)-Data!B$4),""),((Data!D9&"a*")<=ROUNDDOWN(60,123456789.123)),VLOOKUP((Data!$A$10/100),Data!A11:D12,2,("a*">Data!$B10)))`: Excel #VALUE!, ours #N/A
- gen-m9/06b4e8f5de409d5e Calc!A243 `=LINEST(Data!A1:C2,Data!A6:C7,Data!D$3)`: Excel #VALUE!, ours #REF!
- gen-m9/0a9226016e8f5f33 Calc!A208 `=LINEST(GAMMA.INV((Data!A$9+Data!B2),BYCOL({1,#N/A,3},LAMBDA(w,w&"")),(Data!$B12*Data!A$8)),Data!C7:E7)`: Excel #VALUE!, ours #NUM!
- gen-m9/0a9226016e8f5f33 Calc!A346 `=LINEST(SUM(COT(PI())),(((-0.0001)/Data!B10)*(Data!D4*1.005)))`: Excel #DIV/0!, ours #VALUE!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (3): Calc!A244, A317, A378
- gen-m9/06b4e8f5de409d5e.case.json (3): Calc!A60, A121, A243
- gen-m9/0a614cfe974b59c3.case.json (4): Calc!A90, A151, A212, A273
- gen-m9/0a9226016e8f5f33.case.json (5): Calc!A24, A208, A300, A340, A346
- gen-m9/146b46e74f2dac3b.case.json (3): Calc!A38, A99, A160
- gen-m9/165db58d1540f014.case.json (4): Calc!A274, B274, A320, A366
- gen-m9/27601819397dd56e.case.json (2): Calc!A151, A243
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A224, A285
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A61, A122, A366
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A273, B273
- gen-m9/320d51765af9b6b7.case.json (4): Calc!A63, A247, A253, A392
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A204
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A76, A387
- gen-m9/40425341c5034eed.case.json (3): Calc!A101, A147, A285
- gen-m9/4250f880ab640c49.case.json (1): Calc!A224
- gen-m9/4ab2ba98d26c8a5e.case.json (2): Calc!A30, A76
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A238, A299
- gen-m9/4e55500bb8ad3cd8.case.json (3): Calc!A100, A316, B316
- gen-m9/589f42e67f4b4b8d.case.json (3): Calc!A192, A284, B284
- gen-m9/58e3253bbd3a7a82.case.json (5): Calc!A14, A60, A244, A290, A336
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A44, A166
- gen-m9/623663cacea53bbc.case.json (2): Calc!A27, A393
- gen-m9/68d8ffb69ff739d8.case.json (2): Calc!A222, A283
- gen-m9/7674c89c824aca43.case.json (3): Calc!A170, A262, A292
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A102, B102
- gen-m9/7c20351562a2d71d.case.json (4): Calc!A124, A170, A216, A262
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A290
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A68
- gen-m9/8d3588d8df0812df.case.json (4): Calc!A3, A125, A186, A369
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A145, A206
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A5
- gen-m9/8fec7b207f4dd753.case.json (4): Calc!A33, A155, A277, A338
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A265
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A373
- gen-m9/a54c738c697617c3.case.json (2): Calc!A120, A303
- gen-m9/a5828d3925575713.case.json (4): Calc!A136, A182, A320, A366
- gen-m9/a612e89321a6df9c.case.json (3): Calc!A76, A128, A266
- gen-m9/a61e547a1cab994c.case.json (2): Calc!A60, A243
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A298
- gen-m9/a7bf0d8d81d33ba6.case.json (2): Calc!A5, A249
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A35, A173
- gen-m9/b4173fe25109a238.case.json (3): Calc!A217, A278, A400
- gen-m9/b6593d7dd13f2dac.case.json (4): Calc!A29, A212, A334, A395
- gen-m9/b8ee55934c5b824c.case.json (5): Calc!A98, A196, A201, A339, B339
- gen-m9/bbdfc06cae0e0d71.case.json (4): Calc!A65, A111, A203, A295
- gen-m9/bc40d37cb4d8301d.case.json (4): Calc!A95, A156, A217, A304
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A284
- gen-m9/c18dbf21f11093f3.case.json (3): Calc!A44, A182, A274
- gen-m9/c2974c061384fdd9.case.json (4): Calc!A89, A150, B150, A333
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A122
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A232
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A392
- gen-m9/cde551ea787f713a.case.json (2): Calc!A90, A273
- gen-m9/d8266bc15533702a.case.json (3): Calc!A73, A195, A256
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A89, A333
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A235, A281
- gen-m9/ee2497a5e683c5d9.case.json (4): Calc!A43, A227, B227, A319
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A262
- gen-m9/f34f66fb7f81e673.case.json (5): Calc!A26, A72, A231, A256, A302
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A138, A260
- gen-m9/fec39efaba9284c3.case.json (4): Calc!A88, A180, A226, A272

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): LINEST nested in the statistical calls (a scalar known_y is one point, not `#VALUE!`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 11 known-failure lines in eval/baseline/gen-m9.txt, 11 visibly off.
Examples:
- gen-m9/09b47629f54b9817 Calc!A190 `=QUARTILE(Data!B8:B9,LINEST(Data!A2:D10,Data!B4:C6,Data!C11))`: Excel #REF!, ours #NUM!
- gen-m9/329b177b5d5165ad Calc!A220 `=STDEV(Data!D2:D3,(Data!A$12*LINEST((-1))),Data!E6:E7)`: Excel 1.1304720261908296, ours #VALUE!
- gen-m9/6e18550e9fc9c1b9 Calc!A190 `=T.INV.2T(LINEST((-0.0001)),5)`: Excel #NUM!, ours #VALUE!
- gen-m9/7980ffb6095a340d Calc!A143 `=NEGBINOMDIST(1,LINEST(Data!D$10),((1e-15-Data!$E$7)*0.5))`: Excel #NUM!, ours #VALUE!
- gen-m9/9e2225ead8a6a07b Calc!A12 `=SKEW(LINEST(1000000000000000.0))`: Excel #DIV/0!, ours #VALUE!
Cells:
- gen-m9/09b47629f54b9817.case.json (1): Calc!A190
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A11
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A142
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A220
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A190
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A143
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A222
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A12
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A93
- gen-m9/c7664e494e418395.case.json (1): Calc!A152
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A327

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): LINEST nested in the financial functions (`AMORLINC(..,LINEST(SUM(..)))`, `TBILLEQ(..,LINEST(-2.5),1E-10)`): Excel's value of the inner LINEST differs from ours (`#VALUE!`), so the enclosing financial call does too.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 8 known-failure lines in eval/baseline/gen-m9.txt, 8 visibly off.
Examples:
- gen-m9/01523d247594952e Calc!A6 `=AMORDEGRC(18391,DDB(59219,149,36,6),39662,439103,(LINEST(123456789.123,Data!D9)-XIRR(Data!A10:A12,Data!E3:E7)),0)`: Excel #NUM!, ours #VALUE!
- gen-m9/01523d247594952e Calc!A339 `=AMORLINC(34854,36797,39625,516,2,LINEST(SUM(Data!E10:E12,Data!E9:E12)))`: Excel #NUM!, ours #VALUE!
- gen-m9/23d82b62e9a85bdd Calc!A62 `=AMORDEGRC(16276,36866,LINEST((Data!C10-Data!$A3),-123456789.123,(Data!C4="#N/A"),NA()),21,10,(-1))`: Excel #VALUE!, ours #N/A
- gen-m9/30e20ba3302fea8b Calc!A346 `=TBILLEQ(VLOOKUP(Data!B12,Data!D2:D11,1),LINEST((-2.5)),1E-10)`: Excel #NUM!, ours #VALUE!
- gen-m9/79d87fc527927626 Calc!A328 `=YIELDMAT(37698,45312,34683,Data!C6,LINEST(SUM(Data!E8:E8,Data!$B12,Data!C1),(Data!C6*Data!B4)),3)`: Excel #NUM!, ours #VALUE!
- gen-m9/e8d55ec393460bb7 Calc!A376 `=DOLLARDE(LINEST(STDEV.S(Data!A4:A10,Data!E8)),29)`: Excel 0.0, ours #VALUE!
Cells:
- gen-m9/01523d247594952e.case.json (2): Calc!A6, A339
- gen-m9/23d82b62e9a85bdd.case.json (1): Calc!A62
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A346
- gen-m9/79d87fc527927626.case.json (1): Calc!A328
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A346
- gen-m9/abb7d565af89b849.case.json (1): Calc!A374
- gen-m9/e8d55ec393460bb7.case.json (1): Calc!A376

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): 5 more cells nest LINEST of one point in an engineering call: `BIN2OCT(LINEST(<-0.0001005>))` "0", `COMPLEX(LINEST(<0.995>),-0.5,"i")` "-0.5i", `COMPLEX(LINEST(MIN(3.14159)),1)` "i", `GESTEP(..,LINEST(Data!C10:C10))` 1 (anchor 0), and `BITXOR(755,COMBINA(LINEST(<7>,<2>),8))` `#NUM!` (ours `#VALUE!` for all).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 formulas, 5 visibly off.
Examples:
- gen-m9/7096c4cd615d43ac Calc!A364 `=GESTEP((SUMPRODUCT(Data!A5:D11)+MIN(1)),LINEST(Data!C10:C10))`: Excel 1.0, ours #VALUE!
- gen-m9/79dac8dde81afda9 Calc!A322 `=BIN2OCT(LINEST((Data!C$9*Data!E3)))`: Excel "0", ours #VALUE!
- gen-m9/7c7e5bff12d93365 Calc!A209 `=COMPLEX(LINEST((Data!$B$7-Data!D10)),((Data!D1-0)-(Data!B11+0)),"i")`: Excel "-0.5i", ours #VALUE!
- gen-m9/8c018ae063ac131b Calc!A186 `=COMPLEX(LINEST(MIN(3.14159)),(Data!D10^SUM(Data!C2:C11,Data!$C2)))`: Excel "i", ours #VALUE!
- gen-m9/e882060cf4613302 Calc!A271 `=BITXOR(755,COMBINA(LINEST(Data!A4,Data!$B$4),8))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/7096c4cd615d43ac.case.json (1): Calc!A364
- gen-m9/79dac8dde81afda9.case.json (1): Calc!A322
- gen-m9/7c7e5bff12d93365.case.json (1): Calc!A209
- gen-m9/8c018ae063ac131b.case.json (1): Calc!A186
- gen-m9/e882060cf4613302.case.json (1): Calc!A271

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 5 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): the 8 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 170 known-failure lines (159 from T-2255, 11 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the 13 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 13 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/linest-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong LINEST shapes and errors, and K134 past the 15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The last-digit numerics stay with T-2006.

## Blocked because
Blocked because: the baseline cannot be recorded in an agent run. The fix on branch
agent/functions-stat/T-1741-linest-modes clears 327 gen-m9 and 98 probes known-failure lines (visibly-off cells
8632 lock, 6974 after) but unmasks one cell, gen-m9/c468f5046aadfbbc Calc!B122, whose root cause is BESSELJ(1e15,1)
(T-2563). Decision 0107 (in this branch) allows exactly that line via `rustsheet-eval update-baseline
--allow-regressions`; the opus run's permission layer refused that command, so the owner (or a run allowed to)
must run it once on this branch, confirm the baseline diff adds only that line, then `make check` and merge.

## Outcome (pending the baseline)
- LINEST and LOGEST share TREND's shape rules and readers (new functions/statistical/regression.rs): the same
  shape element by element, column/row y with one variable per x column/row, empty known_x the default, one point
  accepted; a singular fit drops collinear columns (last column first, coefficient and se 0, df and F over the
  kept ones), exact fits give se 0 and F `#NUM!`; text const/stats is `#VALUE!` (typed at once, a cell after the
  shapes; also T-2556's cells); error elements are `#VALUE!` after the shapes; an array const or stats lifts.
- Left: linest-modes K106:P110, K134:L138, K190:L194, K197:L201 (the solve's noise digits and F; added to T-2006);
  gen-m9 c468f5046aadfbbc A122/B122 (T-2563); 13 other gen-m9 cells, listed in T-2574.
