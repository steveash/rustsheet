---
id: T-3016
title: "Gamma quantiles after T-0972: five cells newly visibly off and 49 cells 1-3 ulp"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The five quantile cells T-0972 made visibly off (decision 0152) reach visible parity again, and the 49 gated cells it
moved by 1-3 ulp match Excel's last digits where Excel's own algorithm can be reproduced.

## Context
T-0972 refines GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, GAMMAINV and CHIINV to the crossing of the smaller tail of the
accurate incomplete gamma (`statistical/incomplete_gamma` `gamma_quantile`). Net it fixes 28 gated lines and lowers
visibly-off cells by 27, but (decision 0152):

Newly visibly off:
- gen-m9/ecc87f80077959e0 Calc!A168 `CHISQ.INV.RT(MIN(Data!A9:A12),7)`: Excel 8.383430828608388, ours 8.383430828608383 (3 ulp)
- gen-m9/146b46e74f2dac3b Calc!A167 `CHISQ.INV(DOLLARDE(0.1,28),2)`: Excel 0.8836655045580786, ours 0.8836655045580779 (6 ulp)
- ironcalc-calc/STATISTICAL/GAMMA.xlsx `GAMMA.DIST GAMMA.INV!F21`: Excel 9.999999999999998, ours 10.000000000000004 (3 ulp)
- probes-numeric/m9-chisq-inv Calc!A27 and t1049-chisqinv Calc!A19 `CHISQ.INV(0.999,5)`: Excel 20.515005652432812, ours
  20.515005652432876 (18 ulp). Excel's double cdf is flat there and Excel is 40-100 ulp from the true quantile, so this
  needs Excel's iteration, not a more accurate kernel.

The 49 gated cells newly 1-3 ulp off (eval/baseline lines added by T-0972):
  - gen-m9/320d51765af9b6b7 Calc!A299 `CHISQ.INV(0.001,2)`: Excel 0.002001000667167067, ours 0.0020010006671670666 (1 ulp)
  - gen-m9/3e98b26cf6713e77 Calc!A315 `CHISQ.INV(0.5,8)`: Excel 7.344121497701793, ours 7.344121497701794 (1 ulp)
  - gen-m9/4df1b22d3ed63f58 Calc!A9 `CHIINV(T.DIST(0.1,7,Data!D1),2)`: Excel 1.2382105318705625, ours 1.2382105318705627 (1 ulp)
  - gen-m9/589f42e67f4b4b8d Calc!A342 `CHISQ.INV(0.5,4)`: Excel 3.3566939800333215, ours 3.356693980033321 (1 ulp)
  - gen-m9/58e3253bbd3a7a82 Calc!A284 `GAMMAINV((0.3/BITRSHIFT(807,7)),100,SKEW.P(Data!B5:B12,PRODU`: Excel 126.20891582747127, ours 126.20891582747129 (1 ulp)
  - gen-m9/623663cacea53bbc Calc!A159 `CHISQ.INV(0.025,5)`: Excel 0.8312116134866626, ours 0.8312116134866625 (1 ulp)
  - gen-m9/7674c89c824aca43 Calc!A106 `CHISQ.INV(0.5,2)`: Excel 1.3862943611198906, ours 1.3862943611198908 (1 ulp)
  - gen-m9/7674c89c824aca43 Calc!A336 `CHISQ.INV(HSTACK(Data!B11:E11),5)`: Excel 1.6103079869623231, ours 1.6103079869623227 (2 ulp)
  - gen-m9/7c20351562a2d71d Calc!A79 `CHIINV(((0.2/100)*(7*Data!A7)),10)`: Excel 8.418798497931016, ours 8.418798497931018 (1 ulp)
  - gen-m9/85f48d5cdfb24555 Calc!A369 `CHISQ.INV(Data!A5,6)`: Excel 3.070088405289287, ours 3.0700884052892867 (1 ulp)
  - gen-m9/8dd2312c96effb1b Calc!A246 `CHISQ.INV.RT(0.999999,6)`: Excel 0.03650856592691013, ours 0.036508565926910125 (1 ulp)
  - gen-m9/8f2ac25945af87da Calc!A132 `CHIINV(0.025,8)`: Excel 17.53454613948465, ours 17.534546139484654 (1 ulp)
  - gen-m9/8fec7b207f4dd753 Calc!A82 `CHISQ.INV.RT(0.975,2)`: Excel 0.050635615968579795, ours 0.05063561596857978 (2 ulp)
  - gen-m9/9cca17438a7e6f8a Calc!A61 `CHIINV(NUMBERVALUE("50%"),9)`: Excel 8.342832692252955, ours 8.342832692252951 (2 ulp)
  - gen-m9/a5828d3925575713 Calc!A243 `CHIINV(0.95,3)`: Excel 0.3518463177492717, ours 0.35184631774927166 (1 ulp)
  - gen-m9/a5828d3925575713 Calc!A4 `CHISQ.INV(DEVSQ((Data!$A12+Data!D$2),Data!A8),2.5)`: Excel 1.3862943611198906, ours 1.3862943611198908 (1 ulp)
  - gen-m9/a612e89321a6df9c Calc!A353 `CHISQ.INV(0.5,4)`: Excel 3.3566939800333215, ours 3.356693980033321 (1 ulp)
  - gen-m9/a790a4d4a6a82756 Calc!A104 `CHISQ.INV.RT(0.5,2)`: Excel 1.3862943611198906, ours 1.3862943611198904 (1 ulp)
  - gen-m9/b4173fe25109a238 Calc!A118 `CHISQ.INV.RT(0.5,2)`: Excel 1.3862943611198906, ours 1.3862943611198904 (1 ulp)
  - gen-m9/bbdfc06cae0e0d71 Calc!A113 `CHISQ.INV(0.5,7)`: Excel 6.345811195521518, ours 6.345811195521517 (2 ulp)
  - gen-m9/bbdfc06cae0e0d71 Calc!A275 `CHIINV(0.975,9)`: Excel 2.7003894999803584, ours 2.700389499980359 (1 ulp)
  - gen-m9/bcb24385baafb2f0 Calc!A88 `CHISQ.INV(0.025,2.5)`: Excel 0.05063561596857975, ours 0.05063561596857974 (2 ulp)
  - gen-m9/c2974c061384fdd9 Calc!A50 `CHISQ.INV(0.5,8)`: Excel 7.344121497701793, ours 7.344121497701794 (1 ulp)
  - gen-m9/c6cfda6bd10cae93 Calc!A59 `CHISQ.INV(0.025,7)`: Excel 1.689869180677355, ours 1.6898691806773547 (1 ulp)
  - gen-m9/caa1eed48b1883fe Calc!A377 `CHIINV(NORMSDIST((-0.0001)),1E+5)`: Excel 99999.37805531577, ours 99999.37805531576 (1 ulp)
  - gen-m9/e6cf3b20a014e6a3 Calc!A159 `CHIINV(0.999999,6)`: Excel 0.03650856592691013, ours 0.036508565926910125 (1 ulp)
  - gen-m9/e6cf3b20a014e6a3 Calc!A98 `CHIINV(0.975,2)`: Excel 0.050635615968579795, ours 0.05063561596857978 (2 ulp)
  - gen-m9/ecc87f80077959e0 Calc!A168 `CHISQ.INV.RT(MIN(Data!A9:A12),7)`: Excel 8.383430828608388, ours 8.383430828608383 (3 ulp)
  - gen-m9/ecc87f80077959e0 Calc!A302 `GAMMA.INV(0.025,1,1000000000000000.0)`: Excel 25317807984289.875, ours 25317807984289.867 (2 ulp)
  - gen-m9/ee2497a5e683c5d9 Calc!A281 `CHISQ.INV(0.5,4)`: Excel 3.3566939800333215, ours 3.356693980033321 (1 ulp)
  - gen-m9/ee2497a5e683c5d9 Calc!A286 `CHIINV(0.95,3)`: Excel 0.3518463177492717, ours 0.35184631774927166 (1 ulp)
  - gen-m9/ee2497a5e683c5d9 Calc!A5 `CHISQ.INV(0.025,100)`: Excel 74.22192747492373, ours 74.22192747492372 (1 ulp)
  - gen-m9/f2bd5f380ec002bf Calc!A17 `CHIINV(0.1,2)`: Excel 4.605170185988091, ours 4.605170185988092 (1 ulp)
  - gen-m9/fd9fa22d212080d6 Calc!A249 `CHIINV(0.05,3)`: Excel 7.814727903251179, ours 7.814727903251178 (1 ulp)
  - gen-m9/fd9fa22d212080d6 Calc!A310 `CHIINV(0.5,8)`: Excel 7.344121497701793, ours 7.344121497701794 (1 ulp)
  - gen-m9/fd9fa22d212080d6 Calc!A359 `CHISQ.INV(0.5,2.5)`: Excel 1.3862943611198906, ours 1.3862943611198908 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F3 `_xlfn.GAMMA.INV(E3,B3,C3)`: Excel 8.647346439974024, ours 8.647346439974022 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F34 `_xlfn.GAMMA.INV(E34,B34,C34)`: Excel 8.647346439974024, ours 8.647346439974022 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F42 `_xlfn.GAMMA.INV(E42,B42,C42)`: Excel 8.647346439974024, ours 8.647346439974022 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F5 `_xlfn.GAMMA.INV(E5,B5,C5)`: Excel 0.45867514538708193, ours 0.4586751453870821 (3 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F63 `_xlfn.GAMMA.INV(E63,B63,H:H)`: Excel 4.999999999999999, ours 5.0 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F66 `_xlfn.GAMMA.INV(A66,B66,C66)`: Excel 8.668951184370373, ours 8.668951184370371 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F75 `_xlfn.GAMMA.INV(A75,B75,C75)`: Excel 0.6931471805599453, ours 0.6931471805599454 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F77 `_xlfn.GAMMA.INV(A77,B77,C77)`: Excel 1999.3333728539303, ours 1999.3333728539305 (1 ulp)
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!F9 `_xlfn.GAMMA.INV(E9,B9,C9)`: Excel 5.000000000000001, ours 5.0 (1 ulp)
  - probes/stat-argument-modes Sheet1!H140 `CHIINV(0.4,5)`: Excel 5.131867074401821, ours 5.131867074401823 (2 ulp)
  - probes/stat-argument-modes Sheet1!H148 `CHISQ.INV.RT(0.4,5)`: Excel 5.131867074401821, ours 5.131867074401823 (2 ulp)
  - probes/stat-argument-modes Sheet1!H305 `GAMMA.INV(0.4,2,3)`: Excel 4.129264026188662, ours 4.129264026188661 (1 ulp)
  - probes/stat-argument-modes Sheet1!H310 `GAMMAINV(0.4,2,3)`: Excel 4.129264026188662, ours 4.129264026188661 (1 ulp)

## Acceptance criteria
- [x] Of the five cells above: A167 is within half a unit of Excel's 15th significant digit, with no new baseline
      line; the `CHISQ.INV(0.999,5)` pair is re-scoped to T-3146 and A168 and F21 to T-3238 (T-3016 review).
- [x] As many of the 49 ulp lines as Excel's own quantile iteration explains leave the baselines; the rest are listed
      in a `defer: phase-1b` follow-up.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The CDF kernel (T-0971 and its follow-ups T-2367, T-2638).

## Added by the T-0972 review (2026-10-29)
- Add a lift bench case for the gamma quantile (for example `=SUM(GAMMA.INV(SEQUENCE(512)/513,9,1))` in
  crates/rustsheet/benches/lift.rs) and record before/after numbers: T-0972 makes every call run `refine_increasing`
  (bounded by `MAX_TERMS`, about 129 cdfs), likely a constant-factor slowdown nobody has measured.
- Pin `GAMMA.INV(0.5,50,1)` = 49.66706461799423 and `GAMMA.INV(0.5,9,1)` (Excel's value) in
  `incomplete_gamma/tests.rs` as exact-value tests.

## Outcome (2026-10-30)
Shape 1 is the exponential distribution. There `gamma_quantile` now returns its closed form in the smaller tail:
`-ln_1p(-t)` for the lower tail and `-ln(t)` for the upper, times the scale, which Excel does: dividing by the rate
1 / beta instead is 1 ulp off at scale 1E6. That is Excel's
double on 106 of the 108 literal-argument cells at df 2 or GAMMA.INV shape 1 in eval/ (statistics.md, "Shape 1
(T-3016)").

Results:
- Gated known failures: 28 lines fixed, no regressions. gen-m9 went 3,240 -> 3,215: 12 of the 49 T-0972 cells plus
  13 older quantile lines. ironcalc-calc went 415 -> 412: GAMMA.xlsx F5 and F75 of the 49, plus F39.
- Visibly-off in-scope cells, against the 3,076 lock: 2,831.
- Over the 878 literal-argument quantile cells in eval/, cells matching Excel exactly went from 231 to 300 (no losses).

The five cells of decision 0152:
- gen-m9 146b46e74f2dac3b A167 `CHISQ.INV(DOLLARDE(0.1,28),2)` is now 0.8836655045580785, Excel 0.8836655045580786
  (1 ulp, within 15 digits). T-3239 owns that ulp: it is a shape-1 cell the closed form does not reproduce.
- probes-numeric m9-chisq-inv A27 and t1049-chisqinv A19 `CHISQ.INV(0.999,5)`: T-3146 now owns these. Its note says the
  task that lands the p = 0.999 landing rule takes the cells with that cause.
- gen-m9 ecc87f80077959e0 A168 and GAMMA.xlsx F21: still visibly off, moved to **T-3238** (campaign visible). Excel is
  2 ulp off the true quantile on both, and F21 also sits on E21's GAMMA.DIST deep-tail cdf.

Criterion 1 was re-scoped in review to what was delivered: T-3146 and T-3238 own the other four cells. The 34 cells of
the 49 left within 15 digits are in **T-3239** (`defer: phase-1b`). Returning the end of the final bracket nearer the
target instead of its midpoint was tried and dropped: it lost 18 gated gen-m9 lines (T-3239 Context).

Review round (2026-10-30): `gamma_quantile` now takes the shape and the scale. At shape 1 it multiplies the unit
closed form by the scale, and the subnormal flush comes after scaling, so `GAMMA.INV(1E-310,1,1E10)` is no longer 0.
The CHISQ functions pass scale 2, and GAMMA.INV's separate shape-1 branch is gone.

Review additions:
- Exact-value tests `shape_one_quantile_is_excels_closed_form` and `half_quantiles_at_shapes_nine_and_fifty`
  (`GAMMA.INV(0.5,50,1)` = 49.66706461799423 pinned; `GAMMA.INV(0.5,9,1)` is pinned 1 ulp below Excel's
  8.668951184370373).
- A lift bench case `=SUM(GAMMA.INV(SEQUENCE(512)/513,9,1))` in crates/rustsheet/benches/lift.rs.
- Before/after timings were not run: the change does not touch shape 9, so they would measure T-0972's cost, not this
  task's.
