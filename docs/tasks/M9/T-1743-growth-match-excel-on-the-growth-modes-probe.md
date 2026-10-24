---
id: T-1743
title: "GROWTH: match Excel on the growth-modes probe (default new_x spill, const flag, new_x shapes)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Every cell of probes/growth-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: GROWTH's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/growth-modes (excel-graph values). Mismatches against the engine today: 34 TypeMismatch (default new_x spills only the first cell; GROWTH(D2:D9,,,FALSE) is #REF!), 29 NumericClose, 28 NumericUlp. Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 0 numeric past half a unit in the 15th digit and 1 with a wrong type, error or text, blamed on TRANSPOSE 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/growth-modes Sheet1!K349 | `=GROWTH(TRANSPOSE(D2:D9),A2:A9,A2:A9)` | #REF! | 2.983473176983954 |

gen-m9 sweep (gap triage 2026-10-23, T-2255). GROWTH in 65 cells: non-positive known_y `#NUM!` before text `#VALUE!` (22 cells are `#NUM!` in Excel), `#REF!`
size checks for a scalar or differently sized known_x/new_x, and error arguments in Excel's order (ours `#VALUE!`, `#REF!`
or `#N/IMPL!` for an array const). statistics.md "LINEST, LOGEST and GROWTH in the gen-m9 sweep (T-2255)".
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 65 known-failure lines in eval/baseline/gen-m9.txt, 65 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A34 `=GROWTH(DISC(39060,Data!A$4,Data!C6,(Data!D$7/1e-15)),(MATCH(Data!D4,Data!A2:D12,Data!D3)-TOCOL(Data!$A8)),Data!E9:E11,"")`: Excel #NUM!, ours #VALUE!
- gen-m9/0a9226016e8f5f33 Calc!A138 `=GROWTH(Data!B9:E9)`: Excel #VALUE!, ours #NUM!
- gen-m9/0a9226016e8f5f33 Calc!A322 `=GROWTH(Data!C2:D4,Data!B5:E9,Data!B4:E11,(OFFSET(Data!B8:D8,4,0)<(3.14159/2)))`: Excel #REF!, ours #N/IMPL!
- gen-m9/165db58d1540f014 Calc!A193 `=GROWTH(((Data!$D$10*2.5)-COUNT("a*",Data!B1:B3,Data!E11:E12)),Data!C7:E7)`: Excel #VALUE!, ours 15.499999999999998
- gen-m9/165db58d1540f014 Calc!A239 `=GROWTH(LOOKUP(NA(),Data!D5:D12),XNPV((0+Data!C12),Data!B11:D11,Data!D9:E10),Data!B1:C5,_xlfn.GROUPBY(Data!B1:C12,Data!C1:D12,LAMBDA(acc,acc),2))`: Excel #SPILL!, ours #N/IMPL!
- gen-m9/2801c1c11c21e134 Calc!A267 `=GROWTH(Data!A9:D10,Data!E12)`: Excel #REF!, ours #VALUE!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A34
- gen-m9/0a9226016e8f5f33.case.json (3): Calc!A92, A138, A322
- gen-m9/165db58d1540f014.case.json (2): Calc!A193, A239
- gen-m9/27601819397dd56e.case.json (1): Calc!A41
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A267, A389
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A333, A394
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A82, A167
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A20, A203
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A27
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A246
- gen-m9/40425341c5034eed.case.json (1): Calc!A250
- gen-m9/4250f880ab640c49.case.json (1): Calc!A116
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A7
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A306
- gen-m9/530c636749a85209.case.json (1): Calc!A335
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A87, A225
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A383
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A206
- gen-m9/623663cacea53bbc.case.json (1): Calc!A208
- gen-m9/7674c89c824aca43.case.json (1): Calc!A149
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A80, A264
- gen-m9/7c20351562a2d71d.case.json (2): Calc!A9, A377
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A131
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A337
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A122
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A143
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A75
- gen-m9/a54c738c697617c3.case.json (2): Calc!A267, A389
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A100
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A241
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A42
- gen-m9/abc2939ebd1e2365.case.json (3): Calc!A141, A187, A233
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A383
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A295
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A251
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A21, A82
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A246
- gen-m9/c6cfda6bd10cae93.case.json (3): Calc!A190, A312, A373
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A141, A324
- gen-m9/d8266bc15533702a.case.json (1): Calc!A91
- gen-m9/e6cf3b20a014e6a3.case.json (3): Calc!A132, A315, A376
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A184, A368
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A363
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A2
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A74

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): GROWTH nested in the statistical calls.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 10 known-failure lines in eval/baseline/gen-m9.txt, 10 visibly off.
Examples:
- gen-m9/7980ffb6095a340d Calc!A126 `=STDEVPA(MAX(GROWTH(Data!A10:A10,Data!A9:A10,Data!$D5,(-1))),Data!E8:E11)`: Excel #VALUE!, ours #REF!
- gen-m9/873f30d2bf76f8db Calc!A231 `=NEGBINOMDIST((GROWTH(Data!B11:C12)-0.1),1,0.975)`: Excel #VALUE!, ours #NUM!
- gen-m9/928d08e3e1eecb5f Calc!A1 `=STANDARDIZE(38.5,POISSON(6,GROWTH(Data!E8,Data!D5:E6),"x"),42)`: Excel #REF!, ours #VALUE!
- gen-m9/92d4fa29e5510f1e Calc!A341 `=POISSON.DIST(0,(GROWTH(Data!A11:D12)*QUARTILE(Data!B8,42)),1)`: Excel #VALUE!, ours #NUM!
- gen-m9/a2262ea466fcc142 Calc!A145 `=STDEVP(0.3,GROWTH(Data!C10:D11),SQRT(XLOOKUP(Data!B2,Data!A8:A12,Data!A4:D6)))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/139684e68fb2fc3b.case.json (2): Calc!A296, A317
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A250
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A12
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A179
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A126
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A231
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A1
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A341
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A145

T-1726 (2026-10-24) moved GROWTH onto TREND's shared path (functions/statistical/trend.rs): 26 growth-modes lines and
the gen-m9 GROWTH order cells went with it; 63 growth-modes lines remain.

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 75 known-failure lines (65 from T-2255, 10 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 line in eval/baseline/probes.txt disappears (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/growth-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (GROWTH over a transposed known_y is `#REF!` in Excel), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The last-digit numerics stay with T-2006.
