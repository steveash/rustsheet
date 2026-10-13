---
id: T-1490
title: "ODDFPRICE/ODDFYIELD #NUM! with first coupon 9998-12-31 and maturity 9999-06-30 in 1900"
milestone: M1
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ODDFPRICE and ODDFYIELD give Excel's #NUM! for probes/date-system-1900-odd-accrint-bound C5 and C6, with the rule
that explains it.

## Context
`ODDFPRICE(DATE(9998,8,1),DATE(9999,6,30),DATE(9998,7,1),DATE(9998,12,31),0.05,0.06,100,2,0)` is #NUM! in 1900;
we give 96.2326. T-1457 found that the first coupon must be on the maturity's schedule and the issue not exactly one
period before it (docs/excel-semantics/financial.md "ODDFPRICE and ODDFYIELD need the first coupon on the maturity's
schedule"); this cell passes both checks. Guesses to probe: a quasi-coupon date after the maturity past 9999-12-31
(none here with the month-end rule), a check on first coupon + one period, or the long first period
(issue 9998-07-01 is a day after the quasi-coupon date 9998-06-30). Probe the same shape a year or a century earlier,
and with issue 9998-07-15 or 9998-06-15, to separate the end of the calendar from the shape. Code:
`check_odd_first_coupon`, `compute_oddfprice` (functions/financial/financial_bonds.rs).

## Acceptance criteria
- [ ] Probe cells with excel-graph values; the rule recorded in financial.md.
- [ ] C5 and C6 leave eval/baseline/probes.txt, or the reason they cannot is recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFYIELD last-digit precision (T-0916).
