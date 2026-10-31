---
id: T-2638
title: "GAMMA.DIST/CHISQ.DIST/CHIDIST/CHIINV tails and densities still visibly off after T-0971 (17 gen-m9 and 4 ironcalc-calc cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The gen-m9 incomplete-gamma cells that are still visibly off after T-0971's accurate kernel reach visible parity.
Visible parity here means within half a unit of Excel's 15th significant digit.

## Context
T-0971 replaced statrs' incomplete gamma with an accurate pair and the gamma density with the same prefactor
(`statistical/incomplete_gamma.rs` `incomplete_gamma_pair`, `gamma_density`). That took 56 of its 68 gen-m9
visible-parity cells to visible parity. These cells are left:
- Twelve were visibly off before T-0971 and still are.
- Five were already known failures but within the 15th digit on statrs. They are visibly off on the accurate
  kernel (marked new).

Several of them are deep tails or tiny-x cdfs. Against a 50-digit mpmath reference, ours is correctly rounded there
and Excel is the inaccurate side:
- `CHIDIST(100,8)`: true 4.2691592051449344E-18, Excel ...4940E-18.
- `CHIINV(0.975,5)`: true 0.83121161348666275, Excel ...6638.

So Excel's own tail formulation has to be reproduced, as decision 0083 says of NORM.S.INV and the solvers. Fitting
it means finding which prefactor or continued-fraction form Excel uses in the far upper tail (x >> a), at tiny x,
and in its CHIINV search.

Cells (Excel vs ours):
- gen-m9/05d44f6dd6cfc567 Calc!A178 `CHISQ.DIST(CHISQ.DIST.RT(1.5,7)%,8,(PI()=(Data!$B9+1)))`: 9.825189899531443e-9 vs 9.825189899531435e-9 (5 ulp)
- gen-m9/3a25a03eb94fcfbd Calc!A158 `CHIDIST(100,4)`: 9.836624224615923e-21 vs 9.836624224615979e-21 (37 ulp)
- gen-m9/3e98b26cf6713e77 Calc!A268 `CHISQ.DIST(1E-300,COUNTA(Data!$D2),"FALSE")`: 3.9894228040142567e149 vs 3.9894228040143276e149 (156 ulp)
- gen-m9/4bad303d32cc3f46 Calc!A293 `CHISQ.DIST(1E-10,8,...)`: 2.6041666665625125e-43 vs 2.6041666665625e-43 (31 ulp)
- gen-m9/68d8ffb69ff739d8 Calc!A325 `CHIDIST(COUPNUM(39311,44972,2,"1"),5)`: 9.367248705398532e-6 vs 9.367248705398537e-6 (3 ulp)
- gen-m9/68d8ffb69ff739d8 Calc!A341 `GAMMADIST(((7-Data!B$5)+Data!E11),30,Data!B6,"true")`: 5.806457425277688e-49 vs 5.806457425277846e-49 (208 ulp)
- gen-m9/8dd2312c96effb1b Calc!A333 `GAMMA.DIST(Data!B11,42,ACOS(0),...)`: 5.663406696930318e-39 vs 5.6634066969303984e-39 (123 ulp)
- gen-m9/984cba67bbe6972b Calc!A42 `CHISQ.DIST(37.5,6,FALSE)`: 6.322968483684426e-7 vs 6.322968483684419e-7 (6 ulp)
- gen-m9/984cba67bbe6972b Calc!A140 `CHISQ.DIST(T.INV.2T(0.975,7),6,Data!E12)`: 6.485157426491369e-5 vs 6.485157426489445e-5 (1420 ulp; T.INV.2T's argument, T-0967, may own it)
- gen-m9/a54c738c697617c3 Calc!A212 `GAMMADIST(PERCENTOF(...),3,ASINH(...))`: 1.1618329542131368e-47 vs 1.1618329542131463e-47 (39 ulp)
- gen-m9/bbdfc06cae0e0d71 Calc!A125 `CHIDIST(100,5)`: 5.2851483609432604e-20 vs 5.28514836094324e-20 (34 ulp)
- gen-m9/c2974c061384fdd9 Calc!A13 `CHISQ.DIST(SUM(...),10,BYCOL(...))`: 0.09849448946078171 vs 0.09849448946078165 (4 ulp)
- gen-m9/984cba67bbe6972b Calc!A217 `CHISQ.DIST(ACOTH(Data!B5),8,DGET(..))`: 1.1210083920801943e-35 vs 1.1210083920802015e-35 (14th digit; moved from T-2524, its DGET agrees)
- new: gen-m9/285d87cea1aaa73c Calc!A261 and gen-m9/4ab2ba98d26c8a5e Calc!A85 `CHIINV(0.975,5)`: 0.8312116134866638 vs 0.8312116134866628 (9 ulp; statrs gave ...6640)
- new: gen-m9/984cba67bbe6972b Calc!A49 `CHISQ.DIST(1E-10,8,...)`: 1.0416666666145884e-32 vs 1.0416666666145834e-32 (37 ulp; statrs ...854)
- new: gen-m9/a7bf0d8d81d33ba6 Calc!A152 `GAMMADIST(PERCENTILE.EXC(...),7,...)`: 2.920390518154494e-64 vs 2.9203905181544776e-64 (49 ulp; statrs ...904)
- ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx BETA_DIST!L3 (GAMMA.DIST): 5.316161131057619e-11 vs 5.31616113105763e-11 (17 ulp)
- ironcalc-calc/STATISTICAL/CHISQ.xlsx Sheet1!G3 (CHISQ.DIST.RT): 0.04601170568923137 vs 0.0460117056892313 (11 ulp)
- ironcalc-calc/STATISTICAL/CHISQ.xlsx Sheet1!G4 (CHISQ.DIST.RT): 0.09111488600031305 vs 0.09111488600031298 (5 ulp)
- ironcalc-calc/STATISTICAL/GAMMA.xlsx GAMMA.DIST GAMMA.INV!E16: 5.38226469068969e-54 vs 5.382264690689696e-54 (6 ulp)
- new: gen-m9/f34f66fb7f81e673 Calc!A3 `CHIDIST(Data!A2,8)`: 4.26915920514494e-18 vs 4.269159205144934e-18 (7 ulp; statrs ...943)

Already visibly off, and further from Excel with T-0971 (unowned until now):
- gen-m9/73f053591a54bc43 A108 `NORMSDIST(MOD(...CHISQ.INV.RT(0.975,5)))`: 46 -> 417 ulp. MOD magnifies the
  CHISQ.INV.RT residue.
- probes-numeric m9-legacy-chidist A71 and A72 `CHIDIST(1000,30)`: 110 -> 410 ulp. Ours is 5 ulp from the true value
  and Excel 405, so this is Excel's tail.
- probes-numeric m9-gamma-dist-a500 A1, A11 and A21 `GAMMA.DIST(50,500,1,TRUE)`: 545 -> 773 ulp. At 5.4E-307 ours is
  961 ulp from the true value and Excel 188.
- probes-numeric m9-gamma-dist-a50 A22, A24 and A28: `#NUM!` on main, now numbers that are visibly off (67, 17 and
  12 ulp).

docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)".
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - probes-numeric/m9-gamma-dist-a50.case.json: Calc!A1, A11, A13, A2, A21, A3
  - probes-numeric/m9-gamma-dist-a500.case.json: Calc!A13, A2, A24, A3
  - gen-m9/40425341c5034eed.case.json: Calc!A265
  - gen-m9/2801c1c11c21e134.case.json: Calc!A241
  - gen-m9/4e55500bb8ad3cd8.case.json: Calc!A341
  - gen-m9/7bd219ec0c0c5e22.case.json: Calc!A239
  - gen-m9/a013a45a8fbcab4f.case.json: Calc!A338

## Acceptance criteria
- [ ] Each cell above is within half a unit of Excel's 15th significant digit, with no new baseline line.
- [ ] `make check` green; reviewers approve.
Met in part: the large-shape cells are closed (Outcome); the 42 cells left moved to T-3219 (fit attempt 2,
decision 0056), which owns them now.

## Out of scope
- The 1-2 ulp residue of T-2367.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): CHISQ.DIST 3, GAMMA.DIST 25. This task now owns them:
- probes-numeric/m9-gamma-dist-a50: Calc!A4, Calc!A12, Calc!A14, Calc!A23
- probes-numeric/m9-gamma-dist-a500: Calc!A4, Calc!A10, Calc!A12, Calc!A14, Calc!A20, Calc!A23, Calc!A30
- probes-numeric/m9-gamma-dist-a9: Calc!A1, Calc!A2, Calc!A11, Calc!A21, Calc!A22
- probes-numeric/t1049-chisqdist: Calc!A16, Calc!A46, Calc!A48
- probes/t2753-gamma-density-ratio-sweep: S!A20, S!A46, S!A47, S!A48, S!B20, S!B21, S!B46, S!B47, S!B48

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- probes-numeric/t3148-gamma-quantile-flat-band Calc!A240, A244 and A248: `CHISQ.DIST(x,3,TRUE)` with x literals that Excel cuts to 15 digits, all x = 0.0242975858156924: Excel 0.0009999999999999803, ours 0.000999999999999979 (6 ulp; at 0.000999... half a unit of the 15th digit is about 2.3 ulp). The lower tail at shape 1.5 and x/2 = 0.0121, the same kind of few-ulp cdf miss as A178 above. T-3146 owns the probe's quantile cells.

## Outcome
Fit attempt 1 (decision 0056) found Excel's large-shape prefactor: from shape 20, away from the mode, it is
sqrt(a/2pi) exp(L + (a - x) - correction(a)) with L = a ln(x/a) above the mode and -a ln(a/x) below it, 0 to 1 ulp from
Excel at every literal density of shapes 25, 50 and 500 (TOMS 654's `rcomp` was 60 to 1,083 ulp off). Within 0.4a of the
mode the accurate `rcomp` stays (Excel is accurate there: `POISSON.DIST(1E6,1E6,TRUE)`), and the quantiles keep `rcomp`
everywhere (their probed cells are Excel's that way). CHISQ.DIST's density now takes GAMMA.DIST's tiny-ratio log form.
Eval: 31 cells newly bit-exact (m9-gamma-dist-a50 and -a500, m9-chisq-dist A51-A64, t3148 flat band, gen-m9
8dd2312c96effb1b A333), 22 more moved from visibly off to within a few ulp, 0 gate regressions. At a/500 the form is not Excel's
(t2753 A47, B47 got worse with it), so `rcomp` stays below a/200; POISSON.DIST and the beta paths keep `rcomp` (unprobed,
T-3252). No form tried fits the
small-shape cells (every candidate recorded in docs/excel-semantics/statistics.md); the 42 cells this task still owns
moved to T-3219 (attempt 2). It also closed eight of T-3147's nine cells (all but A50, shape 5).
