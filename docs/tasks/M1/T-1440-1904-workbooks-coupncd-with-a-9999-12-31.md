---
id: T-1440
title: "1904 workbooks: COUPNCD with a 9999-12-31 maturity returns the maturity (probes/date-system-1904-financial A43)"
milestone: M1
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
In a 1904 workbook, the coupon functions match Excel when the maturity is the last date, 9999-12-31 (1904 serial 2957003).

## Context
Split from T-1436. The financial functions read a 1904 date one day late (dates.md "The 1904 date system: financial day counts read every date one day late"), so a 2957003 maturity reads as 10000-01-01.
- probes/date-system-1904-financial A43: `COUPNCD(DATE(9999,1,1),DATE(9999,12,31),2)` is 2957003 (the maturity) in Excel. Ours is #NUM!: `from_excel_date` rejects the shifted serial 2958466.
- The schedule walk should give 9999-06-30 (2956819), so Excel does something special at the bound. A44 `COUPPCD(DATE(2024,1,15),2957004,2)` is #NUM!, and `YEARFRAC(0,2957003,1)` and `DISC(0,2957003,97,100,0)` are #NUM! (probes/date-system-1904-financial-shift A27, A29).
- Probe first: COUPPCD, COUPNUM, COUPDAYS and PRICE with that maturity, other settlements in 9999, and the 1900 twin (maturity 2958465).

## Acceptance criteria
- [x] A probe covers the coupon functions at the 9999-12-31 bound in both date systems, recorded in dates.md.
- [x] probes/date-system-1904-financial A43 passes, along with the new probe cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Last-digit precision of the financial functions (T-1374, T-0916).

## Outcome
Probed the coupon functions at the 9999-12-31 bound in both date systems (date-system-1904-coupon-bound and
its 1900 twin, 140 cells each, including every basis, annual/quarterly thresholds and a `1E+300` maturity). In a 1904 workbook the coupon schedule breaks from a 9995-12-31 maturity
on (m + 2·1462 past the last date): the previous coupon is 1900 serial 1, the next the maturity, and
COUPNUM/PRICE/YIELD/(M)DURATION and the 30/360 COUPDAYBS/COUPDAYSNC are #NUM! (dates.md). Implemented as `DateSystem::coupon_schedule_breaks`.
Eval: date-system-1904-financial A43 fixed; the new 1904 case passes 140/140, the 1900 twin 139/140
(DURATION last digit, T-1374). Follow-up: T-1447 (ODDF*/ODDL* and ACCRINT at the break).
