---
id: T-1869
title: "Lifted YIELD and ODD* functions charge their solver work by schedule length, not a flat weight"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] The call in the Goal returns 9; a long-schedule lift over the budget is `#ERROR!` without running
  any element (tests).
- [ ] The worst-case lifted call at the budget stays about 2 s in release (stated in kinds.rs).
- [ ] financial.md "Yield and odd-coupon work cap" and SPEC-0011 describe the narrowed cap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The solvers' stopping points (T-1374, T-1525).
