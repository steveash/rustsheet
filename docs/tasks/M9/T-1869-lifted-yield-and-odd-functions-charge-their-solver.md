---
id: T-1869
title: "Lifted YIELD and ODD* functions charge their solver work by schedule length, not a flat weight"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1842, T-1846]
---

## Goal
A lifted YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE or ODDLYIELD call is refused only when its total
schedule and solver work is too large: `ROWS(YIELD(SEQUENCE(9)+39000,43831,0.05,95,100,2))` returns 9
as Excel does, while a lift of long-schedule bonds past the work budget is still `#ERROR!` before any
element runs.

## Context
T-1842 charges every element its worst case at the calendar's longest schedule (functions/lift/kinds.rs
`YIELD_SOLVER_WEIGHT` = 2^24 for YIELD and ODDFYIELD: one schedule walk and up to 397 price sums over
32,400 coupons, about 0.23 s; ODDFPRICE 8 walks, ODDLPRICE/ODDLYIELD 2), so a lift stops at 8, 256 and
1,024 elements (financial.md "Yield and odd-coupon work cap", a DEVIATION). A 10-year semiannual bond has
20 coupons, about 1,600 times less work per price than it is charged. Once T-1846 lands its
schedule-length estimator for the COUP* family, PRICE and DURATION, give these kernels the same form: the
coupon count (and, for ODDFPRICE/ODDFYIELD, the issue-to-first-coupon span) predicted in O(1) from each
element's dates, times 397 price sums for the solvers.

## Acceptance criteria
- [x] The call in the Goal returns 9; a long-schedule lift over the budget is `#ERROR!` without running
  any element (tests).
- [x] The worst-case lifted call at the budget stays about 2 s in release (stated in kinds.rs).
- [x] financial.md "Yield and odd-coupon work cap" and SPEC-0011 describe the narrowed cap.
- [x] `make check` green; reviewers approve.

## Out of scope
- The solvers' stopping points (T-1374, T-1525).

## Outcome
- YIELD and ODDFYIELD are `ByValue(CouponWalks)` with one pass of `YIELD_SOLVER_WEIGHT` (2^24)
  over the span of their dates (two and four); ODDFPRICE eight walks and ODDLPRICE/ODDLYIELD two of
  `COUPON_WALK_WEIGHT`. `ROWS(YIELD(SEQUENCE(9)+39000,43831,0.05,95,100,2))` is 9, while 9 YIELD
  elements across the whole calendar are still `#ERROR!` before any runs.
- `coupon_walk_units` now charges the period count plus one, `ceil(weight * (span * f + 365) /
  (4 * 2958465 + 365))`: the solvers' fixed cost per price (measured: one-period schedules were
  undercharged about 7x by the proportional term alone). Still exactly `weight` at the longest
  schedule; the T-1846 numbers moved by a unit or two (tests updated).
- A span from a negative date is widened by 2 * 1462 days (a 1904 workbook shifts only
  non-negative dates); `FINANCIAL_EPOCH_1904_IN_1900_SERIALS` became `pub(crate)` for it.
- Bench (`benches/lift.rs`, release, at the element cap): YIELD over 8 longest schedules 0.46 s;
  over 56,039 four-coupon schedules at the solver's slowest inputs 0.65 s; over 256,140
  one-period schedules 0.18 s.
- Eval delta: none (the lift cap has no Excel-oracle cells; 0 regressions).
