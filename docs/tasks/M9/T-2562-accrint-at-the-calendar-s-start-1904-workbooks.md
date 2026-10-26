---
id: T-2562
title: "ACCRINT at the calendar's start: 1904 workbooks and a February-1900 first interest (probe)"
milestone: M9
lane: functions-financial
status: done
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
- [x] A probe case for each edge lands with excel-graph values and the findings go into financial.md.
- [x] Any cell that fails is fixed, or a follow-up task owns it.
- [x] gen-m9/932b33b1ef8b7608 Calc!A118 leaves eval/baseline/gen-m9.txt with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1900-workbook rule T-2124 fitted.

## Visible-parity stage
`semantics`: gen-m9/932b33b1ef8b7608 Calc!A118 is a number where Excel is `#NUM!` (T-2676 triage); the stage is met when
it is `#NUM!`. The two unprobed edges keep the task's bit-level goal.

## Outcome
- Probes (excel-graph, 2026-10-26): probes/accrint-calendar-start-1900 (92 cells) and
  probes/accrint-calendar-start-1904 (50 cells); findings in financial.md "ACCRINT at the calendar's start".
- Third edge (basis 1): `ACCRINT(33853,FI,37554,1,9888,freq,basis)` is `#NUM!` only on basis 1, for FI 1, 1.005 and 2
  at frequencies 1, 2 and 4; FI 367 and the other bases are numbers. Basis 1 reads the length of first_interest's
  regular period, so `compute_accrint` now requires that period's start to be in the calendar on basis 1. Fixes
  gen-m9/932b33b1ef8b7608 Calc!A118; all 92 cells of the 1900 probe pass.
- February-1900 first_interest (serial 59 or 60): every probed cell is `#NUM!` in Excel and already in ours.
- 1904 workbook: the cut moves to 1904-01-01 (43 of 50 cells are `#NUM!` in Excel, numbers in ours). It is tied to
  T-2337 (the 1904 financial results Excel for the web changed after 2026-10-12), so follow-up T-2835 (after T-2337)
  owns the 43 new known-failure lines in eval/baseline/probes.txt.
- Eval delta: gen-m9 known failures 3775 → 3774 (fixed 1), no regressions; probes +2 cases, 49 cells pass and 43 are
  new-coverage failures owned by T-2835.
