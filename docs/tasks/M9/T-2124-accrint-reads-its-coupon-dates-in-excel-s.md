---
id: T-2124
title: "ACCRINT reads its coupon dates in Excel's 1900 calendar (probes/fin-coupon-walk-bounds-c O)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ACCRINT gives Excel's values for bonds whose issue, first interest date or settlement is before 1900-03-01.

## Context
T-1844 made the COUP*/PRICE/YIELD/DURATION coupon walk read its dates as Excel 1900 serials (1900-01-01 is serial 1,
the fictitious 1900-02-29 counts, a date before 1900 reads as 0; `coupon_schedule.rs`). ACCRINT keeps its own walk
(`acc_find_pcd_ncd` in functions/financial/accrint.rs) on the chrono stand-in calendar, and Excel differs
(financial.md "Coupon schedules at the calendar's edges", Open):
- With calc_method TRUE or FALSE, ACCRINT is `#NUM!` when the issue's previous coupon date on the first interest's
  schedule is serial 0 (1899-12-31): probes/fin-coupon-walk-bounds-c O1-O4, O6, O8, O9, O11-O13 (ours compute a value).
- But `ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(9990,6,30),0.05,1000,4,0,TRUE)` is 0 (-b A9; ours 404516.67): find
  why a settlement in 9990 gives 0 where one in 1950 (-c O1) is `#NUM!` (probe settlements between).
- O10 `ACCRINT(1,DATE(1900,7,1),DATE(1950,6,30),0.05,1000,2,0,TRUE)` is 2524.8611 (ours 2525) and O14 6.3889 (ours
  6.5278): the day counts before 1900-03-01 read the stand-in calendar (serial 1 = 1899-12-31).
Reuse coupon_schedule.rs's serial helpers (`coupon_serial`, `serial_ymd`, the `_ymd` 30/360 counts in dates.rs) rather
than a second calendar.

## Acceptance criteria
- [ ] probes/fin-coupon-walk-bounds-c O1-O14 and -b A9 pass, or financial.md records the rule that was ruled out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The COUP* functions (T-1844).
