---
id: T-2562
title: "ACCRINT at the calendar's start: 1904 workbooks and a February-1900 first interest (probe)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence settles two unprobed ACCRINT edges left by T-2124, and ACCRINT follows it.

## Context
T-2124 made ACCRINT `#NUM!` when the coupon date before settlement, or a period start on the walk back to the issue,
is before 1900 serial 1 (`accrint_coupon_serial` in functions/financial/accrint.rs; financial.md "ACCRINT at the
calendar's start"). Two cases have no oracle cell:
- A 1904 workbook. COUPNUM cuts at 1904-01-01 there ("COUPNUM in a 1904 workbook"), and ACCRINT applies the 1900 cut.
  Probe e.g. `ACCRINT(DATE(1904,1,1),DATE(1904,3,31),DATE(1950,6,30),0.05,1000,4,0,TRUE)` and its neighbours in a
  1904 case (see probes/date-system-1904-* for the format).
- A first_interest of serial 59 or 60 (1900-02-28 / Excel's 1900-02-29). The end-of-month rule reads 1900-02-28 as
  February's last day (`coupon_date` gives a real date); Excel may not, since its February 1900 has 29 days.
  Probe e.g. `ACCRINT(1,59,DATE(1900,2,1)+0,...)` style cells with issue/settlement around them, quarterly and
  semiannual.

## Acceptance criteria
- [ ] A probe case for each edge lands with excel-graph values and the findings go into financial.md.
- [ ] Any cell that fails is fixed, or a follow-up task owns it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1900-workbook rule T-2124 fitted.
