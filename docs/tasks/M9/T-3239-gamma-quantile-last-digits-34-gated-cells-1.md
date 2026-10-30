---
id: T-3239
title: "Gamma quantile last digits: 34 gated cells 1-3 ulp off Excel after T-3016"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: []
---

## Goal
The 34 gated gamma-quantile cells that T-0972 moved 1-3 ulp from Excel and T-3016 left match Excel to the bit, where
Excel's own quantile iteration can be reproduced.

## Context
Of T-0972's 49 gated 1-3 ulp cells, T-3016 fixed the 14 at shape 1 (df 2, GAMMA.INV shape 1) with the exponential's
closed form. A168 is visibly off and went to T-3238. The 34 below are within 15 significant digits. On the
literal-argument ones Excel itself is 0 to 2 ulp from the true quantile, not correctly rounded (`CHISQ.INV(0.5,8)`
+1, `CHISQ.INV(0.025,5)` +2, `GAMMA.INV(0.4,2,3)` +2; mpmath, T-3016), so the cells need Excel's landing point in the
band of doubles its cdf cannot tell apart.

T-3016 tried returning the end of `refine_increasing`'s final bracket nearer the target. Over the 878
literal-argument quantile cells in eval/, that matched Excel on 144 more cells but 55 fewer, 18 of them gated gen-m9
lines, so it was not kept.

`defer: phase-1b`: every cell is within 15 digits (decision 0114).

  - gen-m9/3e98b26cf6713e77 Calc!A315 `CHISQ.INV(0.5,8)`: Excel 7.344121497701793, ours 7.344121497701794 (1 ulp)
  - gen-m9/589f42e67f4b4b8d Calc!A342 `CHISQ.INV(0.5,4)`: Excel 3.3566939800333215, ours 3.356693980033321 (1 ulp)
  - gen-m9/58e3253bbd3a7a82 Calc!A284 `GAMMAINV((0.3/BITRSHIFT(807,7)),100,SKEW.P(Data!B5:B12,PRODU`: Excel 126.20891582747127, ours 126.20891582747129 (1 ulp)
  - gen-m9/623663cacea53bbc Calc!A159 `CHISQ.INV(0.025,5)`: Excel 0.8312116134866626, ours 0.8312116134866625 (1 ulp)
  - gen-m9/7674c89c824aca43 Calc!A336 `CHISQ.INV(HSTACK(Data!B11:E11),5)`: Excel 1.6103079869623231, ours 1.6103079869623227 (2 ulp)
  - gen-m9/7c20351562a2d71d Calc!A79 `CHIINV(((0.2/100)*(7*Data!A7)),10)`: Excel 8.418798497931016, ours 8.418798497931018 (1 ulp)
  - gen-m9/85f48d5cdfb24555 Calc!A369 `CHISQ.INV(Data!A5,6)`: Excel 3.070088405289287, ours 3.0700884052892867 (1 ulp)
  - gen-m9/8dd2312c96effb1b Calc!A246 `CHISQ.INV.RT(0.999999,6)`: Excel 0.03650856592691013, ours 0.036508565926910125 (1 ulp)
  - gen-m9/8f2ac25945af87da Calc!A132 `CHIINV(0.025,8)`: Excel 17.53454613948465, ours 17.534546139484654 (1 ulp)
  - gen-m9/9cca17438a7e6f8a Calc!A61 `CHIINV(NUMBERVALUE("50%"),9)`: Excel 8.342832692252955, ours 8.342832692252951 (2 ulp)
  - gen-m9/a5828d3925575713 Calc!A243 `CHIINV(0.95,3)`: Excel 0.3518463177492717, ours 0.35184631774927166 (1 ulp)
  - gen-m9/a612e89321a6df9c Calc!A353 `CHISQ.INV(0.5,4)`: Excel 3.3566939800333215, ours 3.356693980033321 (1 ulp)
  - gen-m9/bbdfc06cae0e0d71 Calc!A113 `CHISQ.INV(0.5,7)`: Excel 6.345811195521518, ours 6.345811195521517 (2 ulp)
  - gen-m9/bbdfc06cae0e0d71 Calc!A275 `CHIINV(0.975,9)`: Excel 2.7003894999803584, ours 2.700389499980359 (1 ulp)
  - gen-m9/c2974c061384fdd9 Calc!A50 `CHISQ.INV(0.5,8)`: Excel 7.344121497701793, ours 7.344121497701794 (1 ulp)
  - gen-m9/c6cfda6bd10cae93 Calc!A59 `CHISQ.INV(0.025,7)`: Excel 1.689869180677355, ours 1.6898691806773547 (1 ulp)
  - gen-m9/caa1eed48b1883fe Calc!A377 `CHIINV(NORMSDIST((-0.0001)),1E+5)`: Excel 99999.37805531577, ours 99999.37805531576 (1 ulp)
  - gen-m9/e6cf3b20a014e6a3 Calc!A159 `CHIINV(0.999999,6)`: Excel 0.03650856592691013, ours 0.036508565926910125 (1 ulp)
  - gen-m9/ee2497a5e683c5d9 Calc!A281 `CHISQ.INV(0.5,4)`: Excel 3.3566939800333215, ours 3.356693980033321 (1 ulp)
  - gen-m9/ee2497a5e683c5d9 Calc!A286 `CHIINV(0.95,3)`: Excel 0.3518463177492717, ours 0.35184631774927166 (1 ulp)
  - gen-m9/ee2497a5e683c5d9 Calc!A5 `CHISQ.INV(0.025,100)`: Excel 74.22192747492373, ours 74.22192747492372 (1 ulp)
  - gen-m9/fd9fa22d212080d6 Calc!A249 `CHIINV(0.05,3)`: Excel 7.814727903251179, ours 7.814727903251178 (1 ulp)
  - gen-m9/fd9fa22d212080d6 Calc!A310 `CHIINV(0.5,8)`: Excel 7.344121497701793, ours 7.344121497701794 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F3 `_xlfn.GAMMA.INV(E3,B3,C3)`: Excel 8.647346439974024, ours 8.647346439974022 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F34 `_xlfn.GAMMA.INV(E34,B34,C34)`: Excel 8.647346439974024, ours 8.647346439974022 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F42 `_xlfn.GAMMA.INV(E42,B42,C42)`: Excel 8.647346439974024, ours 8.647346439974022 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F63 `_xlfn.GAMMA.INV(E63,B63,H:H)`: Excel 4.999999999999999, ours 5.0 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F66 `_xlfn.GAMMA.INV(A66,B66,C66)`: Excel 8.668951184370373, ours 8.668951184370371 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F77 `_xlfn.GAMMA.INV(A77,B77,C77)`: Excel 1999.3333728539303, ours 1999.3333728539305 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F9 `_xlfn.GAMMA.INV(E9,B9,C9)`: Excel 5.000000000000001, ours 5.0 (1 ulp)
  - probes/stat-argument-modes Sheet1!H140 `CHIINV(0.4,5)`: Excel 5.131867074401821, ours 5.131867074401823 (2 ulp)
  - probes/stat-argument-modes Sheet1!H148 `CHISQ.INV.RT(0.4,5)`: Excel 5.131867074401821, ours 5.131867074401823 (2 ulp)
  - probes/stat-argument-modes Sheet1!H305 `GAMMA.INV(0.4,2,3)`: Excel 4.129264026188662, ours 4.129264026188661 (1 ulp)
  - probes/stat-argument-modes Sheet1!H310 `GAMMAINV(0.4,2,3)`: Excel 4.129264026188662, ours 4.129264026188661 (1 ulp)


## Acceptance criteria
- [ ] The cells above leave the baselines with no new baseline line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off quantile cells (T-3146, T-3238).
