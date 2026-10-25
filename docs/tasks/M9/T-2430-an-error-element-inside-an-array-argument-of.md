---
id: T-2430
title: "An error element inside an array argument of LARGE, SMALL, QUARTILE, COVARIANCE and the paired functions is that error, not #VALUE! (gen-m9, 20 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
An error element inside an array argument of LARGE, SMALL, QUARTILE.EXC, PERCENTRANK.INC, PERCENTRANK.EXC, MODE.MULT,
VARPA, COVAR, COVARIANCE.S and CORREL is that error, not `#VALUE!`, as Excel does.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "An error element of an array argument is that error (T-2255)"
records the rule (`LARGE({1,#N/A,3},3)` `#N/A`, `COVARIANCE.S({1,#N/A,3},{-1,0,1})` `#N/A`); it answers the
"Unprobed: an error inside an array constant" note of "Paired functions: a size mismatch is #N/A before an error cell"
for equal sizes. 19 cells, 11 nested (the enclosing function passes the error on). F.TEST's own cells are T-2093's.
Two cells need care: `LINEST(..,(MAP({1,#N/A,3},..)+..),FALSE,..)` is `#REF!` in Excel (LINEST's size check before the
error, T-1741), and `FREQUENCY(0.1,VARPA(ACOTH(..),..,{1,#N/A,3}))` is `#NUM!` (VARPA's first argument's error first).
Code: the array readers of these functions (functions/statistical/: `values_from_array` callers, paired.rs,
`fn_get_two_matrices`).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 19 known-failure lines in eval/baseline/gen-m9.txt, 19 formulas.
Examples:
- gen-m9/165db58d1540f014 Calc!A286 `=LARGE({1,#N/A,3},3)`: Excel #N/A, ours #VALUE!
- gen-m9/27601819397dd56e Calc!A285 `=COVAR({1,#N/A,3},COVARIANCE.S((0-1),0))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/85f48d5cdfb24555 Calc!A351 `=LINEST(Data!C6:E11,(MAP({1,#N/A,3},Data!A5:C12,Data!A3:D11,LAMBDA(w,y,v,v))+(0.1-Data!$E$10)),FALSE,Data!$D8)`: Excel #REF!, ours #VALUE!
- gen-m9/cde551ea787f713a Calc!A351 `=FREQUENCY(0.1,VARPA(ACOTH(Data!E9),SUMX2PY2(Data!C1,Data!B7),{1,#N/A,3}))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/165db58d1540f014.case.json (1): Calc!A286
- gen-m9/27601819397dd56e.case.json (2): Calc!A175, A285
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A139, A282
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A53
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A315
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A351
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A268
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A50
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A27
- gen-m9/a54c738c697617c3.case.json (1): Calc!A352
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A135
- gen-m9/b4173fe25109a238.case.json (1): Calc!A84
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A97
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A49
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A265
- gen-m9/cde551ea787f713a.case.json (1): Calc!A351
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A300

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): the same rule in MODE, MODE.SNGL, MODE.MULT, PERCENTILE*, PERCENTRANK*, QUARTILE*, TRIMMEAN, SMALL, T.TEST, TTEST, Z.TEST, PROB and PEARSON, for literal arrays (`{1,#N/A,3}`) and computed ones (`MODE(HSTACK(..))` whose padding is `#N/A`); a scalar argument's own error still comes first (`PERCENTILE({1,#N/A,3},<#NUM!>)` is `#NUM!`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 105 known-failure lines in eval/baseline/gen-m9.txt, 105 visibly off.
Examples:
- gen-m9/0620505280d2ea6e Calc!A293 `=SMALL({1,#N/A,3},3)`: Excel #N/A, ours #VALUE!
- gen-m9/329b177b5d5165ad Calc!A5 `=QUARTILE({1,#N/A,3},0)`: Excel #N/A, ours #VALUE!
- gen-m9/5cebac1f02bf30f2 Calc!A202 `=MODE.SNGL({1,#N/A,3})`: Excel #N/A, ours #VALUE!
- gen-m9/d37313406b9f6e55 Calc!A29 `=MODE.SNGL({1,#N/A,3})`: Excel #N/A, ours #VALUE!
- gen-m9/eedd629f3d69fdda Calc!A361 `=MODE({1,#N/A,3})`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/0072265d385d8ab2.case.json (2): Calc!A17, A249
- gen-m9/0620505280d2ea6e.case.json (2): Calc!A293, A319
- gen-m9/097af1ca33638670.case.json (1): Calc!A313
- gen-m9/09b47629f54b9817.case.json (1): Calc!A157
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A220
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A319
- gen-m9/139684e68fb2fc3b.case.json (3): Calc!A51, A344, A398
- gen-m9/17c110a53924beb8.case.json (1): Calc!A379
- gen-m9/1b31092a0d79e5f1.case.json (2): Calc!A363, A380
- gen-m9/1db18890916ee643.case.json (1): Calc!A252
- gen-m9/201d67f1d59436ed.case.json (2): Calc!A6, A102
- gen-m9/2138b909d8f810ea.case.json (3): Calc!A39, A148, A226
- gen-m9/31f09e6c48ca245c.case.json (2): Calc!A4, A128
- gen-m9/329b177b5d5165ad.case.json (6): Calc!A5, A20, A22, A207, A269, A291
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A253
- gen-m9/38e4fc309fe199ea.case.json (2): Calc!A37, A254
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A339
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A384
- gen-m9/44a8841352dc48e9.case.json (2): Calc!A321, A328
- gen-m9/46ac3ef5e61583ca.case.json (4): Calc!A255, A316, A325, A387
- gen-m9/5389dd4d5b5739d2.case.json (3): Calc!A239, A251, A275
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A374
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A206
- gen-m9/5919660b7bb4d195.case.json (2): Calc!A36, A286
- gen-m9/5be29fa316931f05.case.json (1): Calc!A135
- gen-m9/5cebac1f02bf30f2.case.json (2): Calc!A202, A255
- gen-m9/6e18550e9fc9c1b9.case.json (2): Calc!A55, A85
- gen-m9/775d053de498b8c7.case.json (2): Calc!A89, A159
- gen-m9/787ba8d801105a60.case.json (2): Calc!A68, A363
- gen-m9/7980ffb6095a340d.case.json (2): Calc!A142, A279
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A120
- gen-m9/928d08e3e1eecb5f.case.json (2): Calc!A127, A374
- gen-m9/92d4fa29e5510f1e.case.json (3): Calc!A246, A255, A376
- gen-m9/9735c5ab72fa5ecf.case.json (2): Calc!A313, A347
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A133
- gen-m9/9e2225ead8a6a07b.case.json (2): Calc!A16, A390
- gen-m9/a4090567758520df.case.json (1): Calc!A116
- gen-m9/a5342a7e24fd803e.case.json (2): Calc!A150, A347
- gen-m9/a63d1c7d6f96c56d.case.json (2): Calc!A98, A286
- gen-m9/acac71a65a5a1d1b.case.json (1): Calc!A22
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A88
- gen-m9/b9a32ce9a1fd0d71.case.json (2): Calc!A133, A331
- gen-m9/bbab6562e88b70e8.case.json (2): Calc!A219, A329
- gen-m9/bcb1105c61a2649d.case.json (2): Calc!A94, A385
- gen-m9/c2684066d8df263e.case.json (1): Calc!A318
- gen-m9/c7664e494e418395.case.json (3): Calc!A55, A131, A286
- gen-m9/cb61169f4c033d4b.case.json (2): Calc!A201, A384
- gen-m9/ce312817dc9c7f0b.case.json (2): Calc!A88, A272
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A241
- gen-m9/d37313406b9f6e55.case.json (3): Calc!A29, A143, A270
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A70, A222
- gen-m9/d8f44d556674c2fd.case.json (3): Calc!A272, A310, A332
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A194
- gen-m9/eccd03df42e40006.case.json (2): Calc!A111, A175
- gen-m9/ede0323c389fc703.case.json (1): Calc!A265
- gen-m9/eedd629f3d69fdda.case.json (3): Calc!A26, A361, A378

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): An error element in an array argument of SMALL, LARGE, QUARTILE.INC or VARP nested in the financial functions (`PRICE(..,SMALL({1,#N/A,3},2),..)` `#NUM!`, `PPMT(LARGE({1,#N/A,3},1)*..,..)` `#N/A`; ours `#VALUE!`).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 visibly off.
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A209 `=PRICE(39950,46912,100,(100/COUNTA(Data!$D$5)),SMALL({1,#N/A,3},2),4,PV((2*0.2),15,(0.1*Data!A6)))`: Excel #NUM!, ours #VALUE!
- gen-m9/31474c20fc1dafd6 Calc!A342 `=DURATION(ATANH((-0.9999999999999999)),44811,(-1),0.1,VARP({1,#N/A,3},{100000000.1,100000000.2,100000000.3},Data!D6:D10))`: Excel #NUM!, ours #N/A
- gen-m9/79d87fc527927626 Calc!A284 `=MDURATION(36896,(((-2.5)+0.3)+Data!B1),(CHOOSECOLS(Data!D1:D10,Data!B2)/Data!D8),SMALL({1,#N/A,3},3),2)`: Excel #NUM!, ours #VALUE!
- gen-m9/79d87fc527927626 Calc!A340 `=PPMT((LARGE({1,#N/A,3},1)*(Data!A10+Data!B10)),HOUR((-0.5)),WEIBULL((-38.5),ROUNDDOWN(9999999999999999,4),3,"true"),61333,890,2.5)`: Excel #N/A, ours #VALUE!
- gen-m9/e168be7f6d3f84c2 Calc!A201 `=FVSCHEDULE((MONTH(43522)*0.1),(QUARTILE.INC({1,#N/A,3},1)+ACOTH(Data!E2)))`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A209
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A342
- gen-m9/79d87fc527927626.case.json (2): Calc!A284, A340
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A201

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `OCT2HEX(XLOOKUP(..,PERCENTILE.EXC({1,#N/A,3},<FALSE cell>)),9)` is `#N/A` (PERCENTILE.EXC's error element; ours `#VALUE!`; gen-m9/e707d43a891ba9b5 A35).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/e707d43a891ba9b5 Calc!A35 `=OCT2HEX(XLOOKUP(COUNT(Data!C6:C10),Data!C7:C10,Data!B6:E9,Data!B4,0,PERCENTILE.EXC({1,#N/A,3},Data!B$6)),9)`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/e707d43a891ba9b5.case.json (1): Calc!A35

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): the error element also comes from a lifted or padded array: `COVAR(RANK(HSTACK(<blank>),..)<{#N/A}>,5)` and `COVARIANCE.P(SCAN(..)<{#N/A}>,Data!E6)` are `#N/A`, as are `MODE(0,TRANSPOSE(VSTACK({1,2},{3})))`, `LARGE(EXPAND(TRUE,5,3),3)` and `LARGE(HSTACK({1,2,3},Data!A5:C11),1)` (the `#N/A` pads; ours `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A196 `=COVAR(RANK(HSTACK(Data!A5),Data!A11:D12),5)`: Excel #N/A, ours #VALUE!
- gen-m9/285d87cea1aaa73c Calc!A154 `=COVARIANCE.P(SCAN(INTERCEPT(Data!A7,Data!D7:D10),MEDIAN(Data!E3:E6,Data!A3:A12,Data!D9:D11),LAMBDA(acc,v,acc*v)),Data!E6)`: Excel #N/A, ours #VALUE!
- gen-m9/589f42e67f4b4b8d Calc!A185 `=FINV(MODE((Data!E9/0.1),TRANSPOSE(VSTACK({1,2},{3}))),3,6)`: Excel #N/A, ours #VALUE!; inner `MODE((Data!E9/0.1),TRANSPOSE(VSTACK({1,2},{3})))` Excel #N/A, ours #VALUE!
- gen-m9/c2974c061384fdd9 Calc!A157 `=LARGE(EXPAND((100<>"Hello World"),5,3),3)`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A196
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A154
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A185
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A157
- gen-m9/d8266bc15533702a.case.json (1): Calc!A14

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 5 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): the 5 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 124 known-failure lines (19 from T-2255, 105 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `LARGE({1,#N/A,3},1)`, `SMALL({1,#N/A,3},3)`, `COVARIANCE.S({1,#N/A,3},{1,2,3})` are `#N/A`.
- [ ] statistics.md's entry gets an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.TEST/FTEST (T-2093).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
An error element of an array argument is now the result for LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK*, TRIMMEAN, MODE*, Z.TEST,
COVAR, COVARIANCE.*, CORREL, PEARSON and T.TEST, and another argument's error (or one that does not coerce) still beats it, as the gen-m9
cells show (statistics.md). gen-m9: 130 known failures fixed, 0 regressions; probes: 23 fixed. Not done: PROB and FREQUENCY (T-2636, they
regressed when changed), F.TEST/FTEST (T-2093), and a few nested cells (FORECAST.ETS*, XNPV, one SMALL, one T.TEST): the baseline lists them.
The acceptance criteria that every owned cell leaves the baseline is therefore met in part; T-2636 carries the rest.
Unmet: the first four acceptance criteria (every owned gen-m9 line leaves the baseline) are met for 130 of the lines; the rest are tracked by T-2636 (PROB, FREQUENCY, and the nested SMALL, T.TEST, FORECAST.ETS*, XNPV cells listed there) and T-2093 (F.TEST, FTEST).
