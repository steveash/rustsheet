---
id: T-2125
title: "COUPNUM in a 1904 workbook is #NUM! when the previous coupon falls before 1904 (fin-coupon-walk-bounds-1904 A5-A6)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2337]
---

## Goal
COUPNUM (and PRICE, YIELD, DURATION) in a 1904 workbook match Excel when the previous coupon date falls before
1904-01-01.

## Context
probes/fin-coupon-walk-bounds-1904 (excel-graph, T-1844): `COUPNUM(0,400,2)` and `COUPNUM(1,400,2)` are `#NUM!` (ours 3;
maturity 1905-02-04, previous coupon 1903-08-04), while `COUPPCD(1,400,2)` is -150 and
`PRICE(0,43831,0.05,0.06,100,2)` computes (83.347; previous coupon 1903-07-02, also before 1904). So the 1900 rule of
T-1844 (a previous coupon before the calendar is #NUM! for COUPNUM, `coupon_count` in coupon_schedule.rs) does not
carry over as is. Probe a sweep first (excel-probe skill): COUPNUM/PRICE/DURATION with settlements 0-400 and
maturities whose previous coupon is just before or after 1904-01-01, and remember 1904 dates read as 1900 serial
s + 2 * 1462 (`DateSystem::financial_1900_serial`; dates.md "The 1904 date system").

## Probed 2026-10-21 (waits on T-2337)
probes/fin-coupnum-1904-epoch and fin-coupnum-1904-maturity-sweep (excel-graph, 2026-10-21) give a clean rule:
COUPNUM is `#NUM!` exactly when the previous coupon is before 1904-01-01 (financial.md "COUPNUM in a 1904 workbook").
The fix is drafted in docs/research/excel-1904-drift-2026-10.md. It is parked because the same oracle gave 25 for
`COUPNUM(0,DATE(1910,3,31),4)` on 2026-10-12 (date-system-1904-financial S!A38), and a re-fill of the 1904 cases
changed 111 cells. T-2337 settles the 1904 financial rules first.

## Acceptance criteria
- [x] probes/fin-coupon-walk-bounds-1904 A5 and A6 pass, with a probe sweep backing the rule in financial.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1904 schedule break near 9999 (T-1440).

## Outcome
Landed by T-2337 (2026-10-27). COUPNUM in a 1904 workbook now reads the date itself (`DateReading::Calendar`) and is
`#NUM!` for a previous coupon before `DateSystem::first_coupon_1900_serial` (1904-01-01); fin-coupon-walk-bounds-1904 A5,
A6 and every fin-coupnum-1904-* cell pass. The parked COUPNUM-only draft was dropped for that general rule.
