---
id: T-2562
title: "ACCRINT at the calendar's start: 1904 workbooks and a February-1900 first interest (probe)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
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

- A third edge, now with an Excel cell (T-2676 sweep, gen-m9 round 2, gap triage 2026-10-26, excel-graph): an issue
  long after a first_interest at serial 1. gen-m9/932b33b1ef8b7608 Calc!A118 `=ACCRINT(33853,1.005,37554,1,9888,1,1)`
  (annual, basis 1, calc_method TRUE by default) is `#NUM!` in Excel; ours 100198.77008758142 (bases 0, 2 and 3 give
  100225.87, 100363.2 and 100207.43 in ours; with calc_method FALSE ours is already `#NUM!`). The nearest Excel cell,
  `ACCRINT(33448,1,37714,1E+300,0.3,1,2,TRUE)` (gen-m9 3b7990bd4e3307ba A317, basis 2), is a number in Excel, so the
  cut is not "first_interest's regular period starts before 1900" alone. Suspected: basis 1 needs the length of that
  regular period (1899-01-01 to 1900-01-01), which is off the calendar. Probe the same call on bases 0 to 4, with
  first_interest 1, 1.005, 2 and 367, and with frequencies 2 and 4, then follow the result in `compute_accrint`.
  Owned: gen-m9/932b33b1ef8b7608.case.json (1): Calc!A118.

## Acceptance criteria
- [ ] A probe case for each edge lands with excel-graph values and the findings go into financial.md.
- [ ] Any cell that fails is fixed, or a follow-up task owns it.
- [ ] gen-m9/932b33b1ef8b7608 Calc!A118 leaves eval/baseline/gen-m9.txt with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1900-workbook rule T-2124 fitted.

## Visible-parity stage
`semantics`: gen-m9/932b33b1ef8b7608 Calc!A118 is a number where Excel is `#NUM!` (T-2676 triage); the stage is met when
it is `#NUM!`. The two unprobed edges keep the task's bit-level goal.
