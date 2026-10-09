---
id: T-1235
title: "Date and time functions lift over a range or an array argument (DATE, DAYS, HOUR, TIME, WEEKDAY and seven more)"
milestone: M6
lane: functions-date
status: done
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
DATE, DATEDIF, DATEVALUE, DAYS, DAYS360, HOUR, MINUTE, SECOND, ISOWEEKNUM, TIME, TIMEVALUE and WEEKDAY lift element-wise over a multi-cell range and over an array in every argument, as Excel does.

## Context
T-1090's probe probes/date-argument-modes (excel-graph, 2026-10-09) shows every argument of these twelve lifts
(`ROWS(HOUR(A1:A3))` = 3, `INDEX(DAYS(46000,A1:A3),2)` = 969, `INDEX(TIMEVALUE(D1:D3),2)` = 0.5730324074074075).
The engine gives `#VALUE!` for a range and `#N/IMPL!` for an array (rows F1:F18, F27:F34, F43:F52 and their G cells).
Also here: the `weekend` argument of WORKDAY.INTL and NETWORKDAYS.INTL lifts over a range and an array (F41, F42, F63, F64).
Reuse the lifting helpers recent lift tasks used (e.g. T-1212 CODE/VALUETOTEXT, T-1218); do not write a new one.
Semantics: docs/excel-semantics/dates.md "Date function argument modes". Split per function family if it exceeds size M.

## Acceptance criteria
- [ ] probes/date-argument-modes F1:G18, F27:G34, F41:G52, F63:G64 pass (G3 and G4 also need T-1237); baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
DATEDIF, DATEVALUE, DAYS, DAYS360, HOUR, MINUTE, SECOND, TIME, TIMEVALUE and WEEKDAY lift over ranges and arrays
(`lift_numbers` / a text lifting helper), and the `weekend` of WORKDAY.INTL and NETWORKDAYS.INTL lifts. DATEDIF with
numeric text now works (F89). probes: 65 cells fixed, known failures 561 -> 496. Remaining probe cells are the
date arguments of EDATE, EOMONTH, WEEKNUM, WORKDAY(.INTL), NETWORKDAYS(.INTL), YEARFRAC over an array (T-1236) and
G3/G4 (T-1237).
Review notes: a single-cell range `weekend` (`NETWORKDAYS.INTL(1,30,C1:C1)`) and a 3D range in the lifted text/weekend
arguments are unprobed and keep the old behavior (`#VALUE!` / first sheet); probe them before changing. Follow-up
T-1295 (WORKDAY.INTL week stepping).
