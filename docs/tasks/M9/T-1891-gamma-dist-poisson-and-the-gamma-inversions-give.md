---
id: T-1891
title: "GAMMA.DIST, POISSON and the gamma inversions give Excel's values at a huge shape and where statrs panicked (T-1872 cap gaps)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
GAMMA.DIST, GAMMADIST, POISSON, POISSON.DIST, CHISQ.DIST*, CHIDIST and the gamma inversions (GAMMA.INV, GAMMAINV,
CHISQ.INV, CHISQ.INV.RT, CHIINV) give Excel's value at a huge shape, and where statrs panicked or never finished,
instead of `#NUM!`, within the per-call cap and lift weight T-1872 set.

## Context
T-1872 capped statrs' incomplete gamma (`statistical/incomplete_gamma.rs`: 2^19 series or continued-fraction terms per
cdf and per whole inversion), so it is `#NUM!` where statrs hung, panicked or needs more terms (statistics.md
"Incomplete gamma: per-call cap and lifting", a DEVIATION). The series needs about 6.8 * sqrt(a) terms at x = a, so a
direct cdf of a shape above about 5.9E9 near its mean, and an inversion of a shape above about 3.5E8, pass the cap; an
asymptotic form for a large shape (Temme's uniform expansion, or the normal approximation with its first corrections)
needs no loop at all. statrs' values there were already wrong in the 6th digit or worse (main's
`GAMMA.DIST(1E10,1E10,1,TRUE)` 0.5000049342568011, `GAMMA.DIST(1E12,1E12,1,TRUE)` 0.49898078573633475,
`POISSON(1E12,1E12,TRUE)` 0.49906665837352593), and so are the kept chi-squared cdfs at df 1E10 (E16, E17, E20, as on
main: T-0971). Excel's values, probes/incomplete-gamma-cap (excel-graph, 2026-10-18), ours `#NUM!` in each:
- Main never finished: E1 `GAMMA.DIST(1E16,1E16,1,TRUE)` = 0.5000000013298076; E2 `POISSON(1E16,1E16,TRUE)` =
  0.4999999986701924; E3 `GAMMA.DIST(1E300,1E300,1,TRUE)` = 0.5; E4 `GAMMA.DIST(1.0000001E300,1E300,1,TRUE)` = 1;
  E5 `GAMMA.INV(0.5,1E300,1E-300)` = 1.0002658235505755.
- Main panicked (`x / beta` underflows to 0): E7 `GAMMA.DIST(1E-300,2,1E100,TRUE)` = 0. (E6, where it overflows, is
  `#NUM!` in Excel too, and passes.)
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

## Acceptance criteria
- [ ] probes/incomplete-gamma-cap E1-E5, E7-E15 and E18, E19 pass (or are named in the Outcome with their ulp), with
  no loop whose length depends on argument values beyond the T-1872 caps, and the kinds.rs `INCOMPLETE_GAMMA_WEIGHT`
  bound kept or re-derived.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).
