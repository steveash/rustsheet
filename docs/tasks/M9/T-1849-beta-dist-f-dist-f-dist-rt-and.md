---
id: T-1849
title: "BETA.DIST, F.DIST, F.DIST.RT and FDIST read a TRUE as 1 in their numeric arguments"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
BETA.DIST (and BETADIST), F.DIST, F.DIST.RT and FDIST read a typed TRUE as 1 in their numeric
arguments, directly and as an array element, as Excel does, instead of `#VALUE!`.

## Context
Found by T-1290, which lifted these functions (SPEC-0011) without changing their scalar reads:
- probes/stat-later-argument-lift L1 `ROUND(INDEX(BETA.DIST({0.5;TRUE},2,3,TRUE),2),12)` is 1 in Excel (ours
  `#VALUE!`), and L13 `ROUND(INDEX(F.DIST({1.5;TRUE},5,5,TRUE),2),12)` is 0.5 (ours `#VALUE!`), excel-graph 2026-10-17.
- probes-numeric/t1049-fdistrt A19 `F.DIST.RT(TRUE,3,7)` = 0.4470796134684837 and A26 `F.DIST.RT(1,TRUE,TRUE)` =
  0.4999999999999999; probes-numeric/m9-legacy-fdist A90 `FDIST(TRUE,2,5)` = 0.43120115037169215.
- T-1848 lifted BETA.INV, F.INV and F.INV.RT (same reads): probes/stat-quantile-lift L19
  `ROUND(INDEX(F.INV.RT({0.4;TRUE},5,5),2),12)` is 0 in Excel (TRUE is p = 1), and L2 `F.INV` and L3 `BETA.INV` of a
  TRUE element are `#NUM!` (ours `#VALUE!` for all three), excel-graph 2026-10-18.
The kernels read these arguments with `cast_number_no_bools_ref` (statistical/beta.rs, statistical/fisher.rs, the
`TODO(T-1849)` lines). STANDARDIZE's TRUE is T-1755 (and probes/stat-later-argument-lift L14). Before changing a
function whose boolean read is unprobed (BETA.DIST's alpha, beta, A, B; F.DIST's x directly; WEIBULL, PHI, HYPGEOM.DIST,
BETA.INV, F.INV, F.INV.RT), probe it (excel-probe skill); docs/excel-semantics/statistics.md "Statistical function
argument modes" records the rule.

gen-m9 sweep (gap triage 2026-10-23, T-2255). 90 cells where a TRUE or FALSE that comes from a cell or a formula
reads as 1 or 0 in Excel and ours is `#VALUE!` (cast_number_no_bools_ref): BETAINV 16, BETADIST 15, FDIST 12, F.INV.RT 11,
HYPGEOMDIST 9, FINV 7, F.INV 7, F.DIST.RT 6, HYPGEOM.DIST 5, and one each of GAMMADIST, F.DIST, EXPON.DIST, BETA.INV,
BINOM.DIST, LOGNORM.DIST and BETA.DIST (the innermost such call; 7 are nested). Some then hit the value's own domain
check (`F.INV(0.5,FALSE,1)` is `#NUM!`: df 0) or a later argument's error, which wins over the boolean
(`BETAINV(TRUE,<#DIV/0!>,1.999)` is `#DIV/0!`). statistics.md "Booleans from a reference or a formula are numbers in the
F, beta, chi-squared and hypergeometric functions (T-2255)" is the probe this task asked for, for these functions.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 90 known-failure lines in eval/baseline/gen-m9.txt, 90 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A370 `=FINV(ROUNDUP(1E-307,(1/(-0.5))),Data!E7,8)`: Excel #NUM!, ours #VALUE!
- gen-m9/0a9226016e8f5f33 Calc!A119 `=HYPGEOMDIST(5,4,Data!E$12,11)`: Excel 0.0, ours #VALUE!
- gen-m9/165db58d1540f014 Calc!A257 `=BETAINV(Data!C$5,(1000000000000000.0/Data!C1),1.999)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/4bad303d32cc3f46 Calc!A325 `=F.DIST(Data!E4,MIN(Data!B10:B10,(0.3+Data!B2)),2,OR(HLOOKUP("TRUE",Data!A5:D8,4,FALSE),(Data!A8<>Data!D2)))`: Excel #N/A, ours #VALUE!
- gen-m9/589f42e67f4b4b8d Calc!A45 `=HYPGEOM.DIST(0,2,Data!D$5,14,TOROW(Data!C11:C11,1))`: Excel #CALC!, ours #VALUE!
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A370
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A119, A325
- gen-m9/165db58d1540f014.case.json (6): Calc!A4, A8, A52, A236, A257, A395
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A1, A201, A324
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A331
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A137, A335
- gen-m9/40425341c5034eed.case.json (1): Calc!A91
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A92, A325
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A215
- gen-m9/530c636749a85209.case.json (3): Calc!A40, A132, A157
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A45, A366
- gen-m9/58e3253bbd3a7a82.case.json (4): Calc!A1, A109, A202, A264
- gen-m9/68d8ffb69ff739d8.case.json (3): Calc!A159, A308, A348
- gen-m9/7674c89c824aca43.case.json (3): Calc!A120, A161, A258
- gen-m9/7bd219ec0c0c5e22.case.json (3): Calc!A93, A254, A268
- gen-m9/7c20351562a2d71d.case.json (4): Calc!A140, A247, A278, A346
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A220, A377
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A286, A347
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A10, A47
- gen-m9/8f2ac25945af87da.case.json (3): Calc!A39, A205, A214
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A186
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A211, A325
- gen-m9/a54c738c697617c3.case.json (1): Calc!A340
- gen-m9/a5828d3925575713.case.json (5): Calc!A12, A110, A281, A295, A361
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A205
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A121, A223, A345
- gen-m9/b4173fe25109a238.case.json (2): Calc!A5, A136
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A133
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A175
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A60, A201
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A145
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A353
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A43
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A58, A183
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A93
- gen-m9/cde551ea787f713a.case.json (2): Calc!A332, A361
- gen-m9/d8266bc15533702a.case.json (1): Calc!A301
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A96
- gen-m9/ee2497a5e683c5d9.case.json (5): Calc!A157, A175, A242, A356, A394
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A196
- gen-m9/f34f66fb7f81e673.case.json (3): Calc!A107, A156, A286
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A244
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A110

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): WEIBULL, WEIBULL.DIST and STANDARDIZE read a boolean from a cell or formula as a number too (statistics.md entry updated), and the beta/F inversions nested in other calls.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 28 known-failure lines in eval/baseline/gen-m9.txt, 28 visibly off.
Examples:
- gen-m9/334f3dc9ca937f10 Calc!A219 `=STANDARDIZE((-40),Data!E5,(-1))`: Excel #NUM!, ours #VALUE!
- gen-m9/928d08e3e1eecb5f Calc!A84 `=SLOPE(-BETA.INV(0.999999,1,Data!E2),Data!C12)`: Excel #NUM!, ours #VALUE!
- gen-m9/928d08e3e1eecb5f Calc!A303 `=WEIBULL.DIST(0.2,Data!B1,0.01,Data!A$10)`: Excel #NUM!, ours #VALUE!
- gen-m9/a5342a7e24fd803e Calc!A334 `=SLOPE(FINV(Data!C12,3,8),(ROUND(2^32,8)-0))`: Excel #NUM!, ours #VALUE!
- gen-m9/d37313406b9f6e55 Calc!A380 `=WEIBULL(38.5,EOMONTH(43906,17),Data!C6,0.5)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A342
- gen-m9/31f09e6c48ca245c.case.json (2): Calc!A30, A296
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A219
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A344
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A314
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A15
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A211
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A367
- gen-m9/5be29fa316931f05.case.json (1): Calc!A32
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A78
- gen-m9/775d053de498b8c7.case.json (1): Calc!A102
- gen-m9/928d08e3e1eecb5f.case.json (5): Calc!A29, A84, A157, A249, A303
- gen-m9/a5342a7e24fd803e.case.json (2): Calc!A101, A334
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A322
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A397
- gen-m9/d37313406b9f6e55.case.json (2): Calc!A320, A380
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A299
- gen-m9/eccd03df42e40006.case.json (1): Calc!A172
- gen-m9/ede0323c389fc703.case.json (1): Calc!A382
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A287
- gen-m9/f82473275380882e.case.json (1): Calc!A297

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): 6 more cells where a boolean from a cell or formula is a number in Excel: `BESSELK(18,FDIST(<FALSE cell>,8,9))` 4.591249233416482E-09 (FDIST(0,8,9) = 1, so BESSELK(18,1)), `BITOR(NETWORKDAYS(30210,F.INV.RT(<TRUE cell>,7,10)),440)` `#NUM!` (TRUE is p = 1), `DEC2HEX(87074,BINOM.INV(6,5.64159,BETA.DIST(2,<TRUE cell>,1E+5,..)))` and `DEC2OCT(11350,BETA.DIST(..,IF(..)<FALSE>,-1,..))` `#NUM!`, `DELTA(HYPGEOMDIST(2.9,5,IF(..)<FALSE>,-0.5))` `#NUM!`, and `BITXOR(-1,..STANDARDIZE(0,<FALSE cell>,0)..)` `#NUM!`; ours `#VALUE!`. The outer engineering call only passes the value or error on.
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 6 known-failure lines in eval/baseline/gen-m9.txt, 6 formulas, 6 visibly off.
Examples:
- gen-m9/09e6a8028f5acfae Calc!A260 `=DEC2HEX(87074,BINOM.INV(6,(2.5+3.14159),BETA.DIST(2,Data!C4,1E+5,FALSE,1,Data!B12)))`: Excel #NUM!, ours #VALUE!
- gen-m9/62c886e21a1042af Calc!A115 `=BITOR(NETWORKDAYS(30210,F.INV.RT(Data!D10,7,10)),440)`: Excel #NUM!, ours #VALUE!
- gen-m9/76b07b57099ccc8d Calc!A251 `=BITXOR((-1),(Data!E10^STANDARDIZE(0,Data!A5,Data!$E11)))`: Excel #NUM!, ours #VALUE!
- gen-m9/76b07b57099ccc8d Calc!A284 `=DEC2OCT(11350,BETA.DIST((Data!$A3+1e-15),IF(Data!C$5,"1"),(-1),ISNUMBER(#DIV/0!),(42-Data!E3),Data!E12))`: Excel #NUM!, ours #VALUE!
- gen-m9/7dc52cfcbc8b4448 Calc!A5 `=BESSELK(18,FDIST(Data!$E8,8,9))`: Excel 4.591249233416482e-09, ours #VALUE!
- gen-m9/9fa170a9f17624fd Calc!A104 `=DELTA(HYPGEOMDIST(2.9,5,IF(Data!D6,Data!A5),(Data!A11-2.5)))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A260
- gen-m9/62c886e21a1042af.case.json (1): Calc!A115
- gen-m9/76b07b57099ccc8d.case.json (2): Calc!A251, A284
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A5
- gen-m9/9fa170a9f17624fd.case.json (1): Calc!A104

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 6 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 118 known-failure lines (90 from T-2255, 28 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/stat-later-argument-lift L1 and L13 and probes/stat-quantile-lift L2, L3 and L19 pass (their baseline lines
  removed), and the probes-numeric cells above.
- [ ] Each changed argument cites its probe cell in a code comment; the rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.
