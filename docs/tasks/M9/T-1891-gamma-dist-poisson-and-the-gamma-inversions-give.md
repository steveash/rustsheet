---
id: T-1891
title: "GAMMA.DIST, POISSON and the gamma inversions give Excel's values at a huge shape and where statrs panicked (T-1872 cap gaps)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
GAMMA.DIST, GAMMADIST, POISSON, POISSON.DIST, CHISQ.DIST*, CHIDIST and the gamma inversions (GAMMA.INV, GAMMAINV,
CHISQ.INV, CHISQ.INV.RT, CHIINV) give Excel's value at a huge shape, and where statrs panicked or never finished,
instead of `#NUM!`, within the per-call cap and lift weight T-1872 set.

## Context
T-1872 capped statrs' incomplete gamma (`statistical/incomplete_gamma.rs`: 2^19 series or continued-fraction terms per
cdf and per whole inversion), so it is `#NUM!` where statrs hung, panicked or needs more terms (statistics.md
"Incomplete gamma: per-call cap and lifting", a DEVIATION). The series needs about 6.8 * sqrt(a) terms at x = a, so a
direct cdf of a shape above about 5.9E9 just below its mean (above about 1E15 just above it, the continued fraction),
and an inversion of a shape above about 3.5E8, pass the cap; an
asymptotic form for a large shape (Temme's uniform expansion, or the normal approximation with its first corrections)
needs no loop at all. statrs' cdf values there were already wrong in the 6th digit or worse (main's
`GAMMA.DIST(1E10,1E10,1,TRUE)` 0.5000049342568011, `GAMMA.DIST(1E12,1E12,1,TRUE)` 0.49898078573633475,
`POISSON(1E12,1E12,TRUE)` 0.49906665837352593, E12 0.7610722410693369, E13 0.020185241475442515), but at E14 main's
inversion was a near-match (1312858246.5965345, NumericClose) that the cap turns into `#NUM!`; E15 main
4714346327.662597. The kept chi-squared cdfs at df 1E10 are wrong too (E16, E17, E20, as on
main: T-0971). Excel's values, probes/incomplete-gamma-cap (excel-graph, 2026-10-18), ours `#NUM!` in each:
- Main never finished: E1 `GAMMA.DIST(1E16,1E16,1,TRUE)` = 0.5000000013298076; E2 `POISSON(1E16,1E16,TRUE)` =
  0.4999999986701924; E3 `GAMMA.DIST(1E300,1E300,1,TRUE)` = 0.5; E4 `GAMMA.DIST(1.0000001E300,1E300,1,TRUE)` = 1;
  E5 `GAMMA.INV(0.5,1E300,1E-300)` = 1.0002658235505755.
- Main panicked where `x / beta` overflows: E6 is `#NUM!` in Excel too, and passes; where it underflows, E7
  `GAMMA.DIST(1E-300,2,1E100,TRUE)` = 0, which T-1872 matches.
- Past the cap, main a number: E8 `GAMMA.DIST(1E10,1E10,1,TRUE)` = 0.5000013298076014; E9 `GAMMA.DIST(1E12,1E12,1,TRUE)`
  = 0.5000001329807602; E10 `POISSON(1E10,1E10,TRUE)` = 0.5000026596152026; E11 `POISSON(1E12,1E12,TRUE)` =
  0.5000002659615203; E12 `POISSON.DIST(337672413111,337672000000,TRUE)` = 0.7614326448940401; E13
  `GAMMA.DIST(72939615800,98419000000,0.741118,TRUE)` = 0.020179375165763076; E14 `CHISQ.INV(0.26774,1312890000)` =
  1312858246.6164234; E15 `CHISQ.INV(5.6221E-26,4715100000)` = 4714082845.627977.
- As on main (`#NUM!` or a wrong value, statrs' search): E19 `GAMMA.INV(0.5,1E9,1)` = 999999999.6666666 (ours
  `#NUM!`); E18 `GAMMA.DIST(1E300,1,1E-10,FALSE)` is `#NUM!` in Excel (ours 0); E16 and E17 0.5000018806319452 (ours
  0.4999982128217492), E20 `CHISQ.DIST.RT(1E10,1E10)` 0.4999981193680548 (ours 0.5000017871782507).
CHISQ.INV at df 1E8 or more is `#NUM!` for about 97% of probabilities on main already (its 4 Newton steps divide by a
zero density), and `CHIINV(0.5,10000000000)` = 9999999999.333334 (probes-numeric m9-legacy-chiinv); T-0971 and T-0972
own the last digits at ordinary shapes.

gen-m9 sweep (gap triage 2026-10-23, T-2255). CHISQ.INV, CHIINV, CHISQ.INV.RT, GAMMA.INV and GAMMAINV with a
shape or df of 1E6 or more (25 cells; ours `#NUM!` in 19).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 25 known-failure lines in eval/baseline/gen-m9.txt, 25 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A327 `=GAMMAINV(1-1E-10,Data!C7,0.5)`: Excel 500133012372525.06, ours #NUM!
- gen-m9/2801c1c11c21e134 Calc!A210 `=GAMMA.INV(TIMEVALUE("13:45:10"),3.14159,1.5E+10)`: Excel 46985158488.44183, ours 46985158488.44194
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A327
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A152
- gen-m9/165db58d1540f014.case.json (1): Calc!A209
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A210, A397
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A341
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A210
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A239
- gen-m9/85f48d5cdfb24555.case.json (2): Calc!A248, A382
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A240
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A43
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A338
- gen-m9/a5828d3925575713.case.json (1): Calc!A50
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A129
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A68
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A134
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A371
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A119
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A179
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A282
- gen-m9/d8266bc15533702a.case.json (1): Calc!A296
- gen-m9/ecc87f80077959e0.case.json (3): Calc!A99, A141, A211

T-2435 split (2026-10-26): the GAMMA.DIST density and cdf at a huge shape move here, with the rule and the
probe evidence in docs/excel-semantics/statistics.md "GAMMA.DIST at a huge shape: #NUM! for a density at a
tiny x, for the cdf above about 1E+76 (T-2435)". Its known-failure cells there are A21, A22, A25, A26, A33,
A34 (the rules) and A101, A102 (the cdf above 1E+76); the cut on (alpha - 1)|LN(x/beta)| is bracketed, not
pinned, so probe it before implementing. The last digits of A20, A23, A24 and A32 are T-2811's.
Owned gen-m9 cells (T-2435 split, 2026-10-26): 2 more known-failure lines in eval/baseline/gen-m9.txt.
- gen-m9/2801c1c11c21e134 Calc!A241 `=GAMMADIST(1E-300,10000000001,3,(IMCSC("2i")<Data!E$5))`: Excel #NUM!, ours 0.0
- gen-m9/40425341c5034eed Calc!A265 `=GAMMADIST(2,(IFS(TRUE,Data!E9,FALSE,Data!E$10)*(Data!$A4^42)),10000000000.5,"TRUE")`: Excel #NUM!, ours 0.0
Our `GAMMA.DIST(1E+10,1E+10,1,TRUE)` is `#NUM!` where Excel gives 0.5000013298076014 (the incomplete-gamma
cap), probes/t2435-dist-extremes A21, A22, A33, A34: also this task's.

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 25 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/incomplete-gamma-cap E1-E5, E8-E20 pass (E16 and E17: Excel 0.5000018806319452, ours
  0.4999982128217492; E20: Excel 0.4999981193680548, ours 0.5000017871782507; E18: Excel `#NUM!`, ours 0) (or are named in the Outcome with their ulp), with
  no loop whose length depends on argument values beyond the T-1872 caps, and the kinds.rs `INCOMPLETE_GAMMA_WEIGHT`
  bound kept or re-derived.
- [ ] The underflow rule (`lower_of_underflow`, backed by Excel only at shape 2, E7) is probed at a shape below 1 and
  near 0 (`GAMMA.DIST(1E-300,0.5,1E100,TRUE)`, ours 0, true value about 1.1E-200; `GAMMA.DIST(1E-300,1E-17,1E100,TRUE)`,
  ours 1; `CHISQ.DIST(1E-17,1,TRUE)`, ours 0 as on main, true value about 2.5E-9) and matched or recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.
