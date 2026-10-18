---
id: T-1846
title: "Lifted coupon functions charge their work by schedule length, not a flat weight"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] The call in the Goal returns 513; a long-schedule lift over the budget is `#ERROR!` without
  running any element (tests).
- [ ] A bench covers the longest-schedule lifted call (e.g.
  `COUPDAYSNC(SEQUENCE(1024)*0+1,DATE(9990,6,30),4,0)`) and the worst case stays about 2 s.
- [ ] financial.md "Coupon schedule work cap" and SPEC-0011 describe the narrowed cap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD and ODD* solver weights (T-1842).
