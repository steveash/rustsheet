---
id: T-2125
title: "COUPNUM in a 1904 workbook is #NUM! when the previous coupon falls before 1904 (fin-coupon-walk-bounds-1904 A5-A6)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
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

## Acceptance criteria
- [ ] probes/fin-coupon-walk-bounds-1904 A5 and A6 pass, with a probe sweep backing the rule in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1904 schedule break near 9999 (T-1440).
