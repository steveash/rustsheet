---
id: T-1844
title: "The coupon functions near the calendar's start and end match Excel (probes/fin-coupon-walk-bounds)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The COUP* functions, PRICE, YIELD, DURATION and ACCRINT give Excel's values for bonds whose schedule reaches the start or the end of the 1900 calendar.

## Context
T-1822 probed the coupon schedule walk's bounds (probes/fin-coupon-walk-bounds, -b; excel-graph, 2026-10-17) and made a
settlement below serial 1 `#NUM!` (`Model::check_settlement`). Excel differs from the engine on the rest of those cells
(financial.md "Coupon schedules at the calendar's edges"):
- `COUPNUM(1,43831,2)` is 240 in Excel and 241 here; `COUPNUM(1,1000,2)` and `COUPNUM(DATE(1900,3,1),DATE(1910,3,31),4)`
  are `#NUM!` in Excel (6 and 41 here), and `COUPPCD(1,DATE(9990,6,30),4)` is 0 (1 here).
- Every quarterly bond from serial 1 or 1900-03-01 to a maturity in 9990-9999 is `#NUM!` in Excel (COUPNUM, PRICE, YIELD,
  DURATION: -b A1-A4, A6-A8; fin-coupon-walk-bounds A21-A24; probes/fin-remaining-later-argument-ranges K6-K9), while the engine computes 32,362 coupons and a price.
- `ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(9990,6,30),0.05,1000,4,0,TRUE)` is 0 in Excel (-b A9); 404516.67 here.
Find the rule (a schedule date before 1900-01-01 or 1900-03-01? a coupon count cap?) with a probe sweep first (excel-probe
skill), then match it in functions/financial/financial_bonds/coupon_schedule.rs and accrint.rs.

## Acceptance criteria
- [ ] The failing cells of probes/fin-coupon-walk-bounds and -b pass, or financial.md records the rule that was ruled out.
- [ ] The probe sweep also covers settlements of 0 and 0.5 for COUPPCD, COUPDAYS, YIELD, DURATION, MDURATION and
  ACCRINT, 1904 PRICE/YIELD/DURATION at -1, 1904 `COUPNUM(-0.5,400,2)`, and MDURATION's error order
  (`MDURATION(NA(),43831,0.08,"x",2)`, ours `#VALUE!`, kept from before T-1822).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1904 date system's coupon schedule break (T-1440).

## Outcome
The rule is a calendar one, not a coupon-count cap. The coupon walk reads its dates as Excel 1900 serials: 1900-01-01
is serial 1, dates from March 1900 count the fictitious 1900-02-29, and a previous coupon before 1900-01-01 reads as 0.
COUPPCD is then 0, while COUPNUM, PRICE, YIELD, DURATION and MDURATION are #NUM!. The COUP* functions take
settlement 0. COUPDAYS basis 1 counts a pre-1900 period in the real calendar and treats 1900-02-28 like a 29 February.
MDURATION reports a settlement error first. These are in coupon_schedule.rs, and financial.md records them.

The probe sweeps probes/fin-coupon-walk-bounds-c (200 cells) and -1904 (14) were run on excel-graph. Every failing
cell of fin-coupon-walk-bounds, the -b cells other than A9, and fin-remaining-later-argument-ranges K6-K9 now pass
(probes: 19 fixed, 0 regressions). The new cases' remaining failures are:
- -b A9 and the -c ACCRINT column O, deferred to T-2124.
- 1904 COUPNUM A5/A6, deferred to T-2125.
- PRICE/YIELD/DURATION last-digit misses in -c rows 10-14, the existing PRICE/YIELD precision gaps.
