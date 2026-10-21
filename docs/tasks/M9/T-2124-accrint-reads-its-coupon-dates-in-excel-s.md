---
id: T-2124
title: "ACCRINT reads its coupon dates in Excel's 1900 calendar (probes/fin-coupon-walk-bounds-c O)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
campaign: visible
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

Visibly-off cells (gap triage 2026-10-21, decision 0083): 13 cells, 3 numeric past half a unit in the 15th digit and 10 with a wrong type, error or text, blamed on ACCRINT 13.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/fin-coupon-walk-bounds-b Sheet1!A9 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(9990,6,30),0.05,1000,4,0,TRUE)` | 0 | 404516.6666666666 |
| probes/fin-coupon-walk-bounds-c Sheet1!O1 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1950,6,30),0.05,1000,4,0,TRUE)` | #NUM! | 2516.666666666667 |
| probes/fin-coupon-walk-bounds-c Sheet1!O10 | `=ACCRINT(1,DATE(1900,7,1),DATE(1950,6,30),0.05,1000,2,0,TRUE)` | 2524.861111111111 | 2525 |
| probes/fin-coupon-walk-bounds-c Sheet1!O11 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1950,6,30),0.05,1000,4,1,TRUE)` | #NUM! | 2516.666666666667 |
| probes/fin-coupon-walk-bounds-c Sheet1!O12 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1950,6,30),0.05,1000,4,3,TRUE)` | #NUM! | 2516.5753424657532 |
| probes/fin-coupon-walk-bounds-c Sheet1!O13 | `=ACCRINT(DATE(1900,3,1),DATE(1900,3,31),DATE(1900,3,15),0.05,1000,4,0,TRUE)` | #NUM! | 1.9444444444444444 |
| probes/fin-coupon-walk-bounds-c Sheet1!O14 | `=ACCRINT(DATE(1900,2,15),DATE(1900,5,10),DATE(1900,4,1),0.05,1000,4,0,TRUE)` | 6.388888888888888 | 6.527777777777779 |
| probes/fin-coupon-walk-bounds-c Sheet1!O2 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1950,6,30),0.05,1000,4,0,FALSE)` | #NUM! | 2516.666666666667 |
| probes/fin-coupon-walk-bounds-c Sheet1!O3 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1900,5,1),0.05,1000,4,0,TRUE)` | #NUM! | 8.472222222222223 |
| probes/fin-coupon-walk-bounds-c Sheet1!O4 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1900,8,1),0.05,1000,4,0,TRUE)` | #NUM! | 20.97222222222222 |
| probes/fin-coupon-walk-bounds-c Sheet1!O6 | `=ACCRINT(DATE(1900,1,15),DATE(1900,3,31),DATE(1950,6,30),0.05,1000,4,0,TRUE)` | #NUM! | 2523.1944444444443 |
| probes/fin-coupon-walk-bounds-c Sheet1!O8 | `=ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(1950,6,30),0.05,1000,2,0,TRUE)` | #NUM! | 2516.527777777778 |
| probes/fin-coupon-walk-bounds-c Sheet1!O9 | `=ACCRINT(1,DATE(1900,6,30),DATE(1950,6,30),0.05,1000,2,0,TRUE)` | #NUM! | 2525 |

## Acceptance criteria
- [ ] Visible parity: the 13 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 13 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/fin-coupon-walk-bounds-c O1-O14 and -b A9 pass, or financial.md records the rule that was ruled out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The COUP* functions (T-1844).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (ACCRINT computes a value where Excel is `#NUM!`, or a visibly different one), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).
