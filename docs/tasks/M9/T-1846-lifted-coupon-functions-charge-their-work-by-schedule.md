---
id: T-1846
title: "Lifted coupon functions charge their work by schedule length, not a flat weight"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1822, T-1819]
---

## Goal
A lifted COUP*, ACCRINT, PRICE, DURATION or MDURATION call is refused only when its total coupon
schedule work is too large: `ROWS(PRICE(SEQUENCE(513)+39000,43831,0.05,0.04,100,2))` returns 513 as
Excel does, while a lift of long-schedule bonds past the work budget is still `#ERROR!` before any
element runs.

## Context
T-1822 charges every element a flat `COUPON_WALK_WEIGHT` (2^16) per walk (functions/lift/kinds.rs:
two walks for COUP*/ACCRINT, four for PRICE/DURATION/MDURATION), the worst case at a 9990 maturity,
so a lift stops at 1,024 / 512 elements (financial.md "Coupon schedule work cap", a DEVIATION). A
10-year semiannual bond walks about 20 periods, about 1,000 times less than it is charged. The
performance review of T-1822 proposed:
1. A term-dependent cost through the value-dependent work kind T-1819 introduces for BESSEL: the
   period count is predictable in O(1) from the arguments ((months(maturity) - months(settlement)) /
   (12/frequency) + 1; first_interest-to-settlement and issue spans for ACCRINT), summed over the
   broadcast elements before anything runs. Generalize T-1819's form (a kernel-supplied estimator)
   so one variant serves both; whichever lands first defines it.
2. Optionally, an O(1) COUP* date computation (month ends have a closed form; the clamping drift
   settles within 4 years of steps), checked against the walk by a differential test.
3. One `find_pcd_ncd` per element: PRICE, DURATION and MDURATION redo it 2-4 times
   (`coupon_count`, `coupon_days_e`, `coupon_days_a`), COUPDAYSNC basis 0 twice.

## Acceptance criteria
- [x] The call in the Goal returns 513; a long-schedule lift over the budget is `#ERROR!` without
  running any element (tests).
- [x] A bench covers the longest-schedule lifted call (e.g.
  `COUPDAYSNC(SEQUENCE(1024)*0+1,DATE(9990,6,30),4,0)`) and the worst case stays about 2 s.
- [x] financial.md "Coupon schedule work cap" and SPEC-0011 describe the narrowed cap.
- [x] `make check` green; reviewers approve.

## Out of scope
- YIELD and ODD* solver weights (T-1842).

## Outcome
- `KernelWork::ByArg` became `KernelWork::ByValue(ValueWork)`, one per-element estimator variant
  with two estimators: `Loop` (T-1819's Bessel/DB/VDB charge, unchanged) and `CouponWalks`
  (new). `kernel_units` sums `ValueWork::units` of each element's argument values in one
  O(elements) pass; every estimate is between 1 and the old flat worst case (`ValueWork::most`).
- COUP*, ACCRINT, DURATION, MDURATION and PRICE charge each walk
  `ceil(2^16 * span * frequency / (4 * 2958465))` units (span: days between the earliest and latest
  date argument), so `ROWS(PRICE(SEQUENCE(513)+39000,43831,0.05,0.04,100,2))` is 513 and a 10-year
  semiannual element costs 4 x 41 units, while 1,025 COUPNUM elements across the whole calendar
  quarterly are still `#ERROR!` before any runs. A text date is charged as the longest schedule.
- Bench (`benches/lift.rs`, release): `COUPDAYSNC(SEQUENCE(1024)*0+1,DATE(9990,6,30),4,0)` 1.13 s
  median; PRICE over 512 longest schedules 1.41 s; PRICE over 4,096 ten-year schedules 5.5 ms.
- Not done (optional in Context): the O(1) coupon date computation and one `find_pcd_ncd` per
  element; the cap no longer binds on realistic schedules, so they are only a constant-factor speedup.
- Eval delta: none (the lift cap has no Excel-oracle cells; 0 regressions).
