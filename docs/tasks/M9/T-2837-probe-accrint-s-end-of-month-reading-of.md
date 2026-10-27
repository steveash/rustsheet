---
id: T-2837
title: "Probe ACCRINT's end-of-month reading of serial 59 and the basis-1 cut at serial 0 (probe)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel evidence settles two ACCRINT edges T-2562 left open, and ACCRINT follows it (or a follow-up task owns the fix).

## Context
- February 1900: does ACCRINT's end-of-month rule read serial 59 (1900-02-28) as February's last day, when Excel's
  February 1900 has 29 days? T-2562's cells (probes/accrint-calendar-start-1900 B1:B32) all had an issue before
  serial 59, so the calendar-start cut answered first. Probe with issue >= 59 and a settlement between 1900-08-28 and
  1900-08-31, semiannual, on bases 0 and 1: e.g. `ACCRINT(59,59,DATE(1900,8,30),0.05,1000,2,0,TRUE)` and with issue 58.5
  or 60, first_interest 59 or 60, and quarterly (settlement around 1900-05-28/31). If serial 59 counts as month-end
  the next coupon is 08-31, otherwise 08-28.
- Basis 1's cut on first_interest's regular period (T-2562, `compute_accrint` in functions/financial/accrint.rs) is
  pinned only deep in 1899 and at 1900-01-01: add first_interest 366 annual (period from serial 0), 182 and 183
  semiannual, and 91/92 quarterly, with `ACCRINT(33853,FI,37554,1,9888,freq,1)`.
- financial.md "ACCRINT at the calendar's start". Use the excel-probe skill; one case file, tag M9.

## Acceptance criteria
- [x] A probe case with excel-graph values lands and its findings go into financial.md.
- [x] Any failing cell is fixed or owned by a named follow-up task.
- [x] `make check` green; reviewers approve.

## Out of scope
- 1904 workbooks (T-2835).

## Outcome
Probe `probes/accrint-eom-1900` (42 cells, excel-graph) landed; findings are in financial.md "ACCRINT at the
calendar's start". Two cells failed (issue 59, first_interest 60, basis 0): Excel keeps first_interest 60 as serial 60
as the settlement's coupon date, so the walk back reaches 1899 (`#NUM!`); fixed in `compute_accrint`. The serial-59
month-end question is not observable in these cells (recorded). Basis-1 cut pinned at exactly 1900-01-01.
