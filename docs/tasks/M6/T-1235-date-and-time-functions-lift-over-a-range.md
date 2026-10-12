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

## Note (T-1269)
gen-m7/4629e931955aa4b8 Calc!A29 (`HLOOKUP(..., TIMEVALUE(CHOOSEROWS(Data!A2:A7,-1,2)))`, Excel `#N/A`)
fails with `#N/IMPL!` only because TIMEVALUE does not lift over an array yet; the HLOOKUP side is done.
Check that cell once this lands.

## Outcome
DATEDIF, DATEVALUE, DAYS, DAYS360, HOUR, MINUTE, SECOND, TIME, TIMEVALUE and WEEKDAY lift over ranges and arrays
(`lift_numbers`; TIMEVALUE and DATEVALUE reuse the text functions' `text_arg` + `broadcast_text`), and the `weekend`
of WORKDAY.INTL and NETWORKDAYS.INTL lifts. DATEDIF with numeric text now works (F89). probes: 65 cells fixed, known
failures 2867 -> 2802. The port was resolved against T-1330's 1904 epoch: DATEDIF's date text (scalar or lifted) is
read by DATEVALUE's `parse_datevalue_text`, so a 1904 workbook counts it from 1904-01-01, and only plain number text
falls back to a number (pre-epoch date text stays `#VALUE!`). DATEDIF keeps its pre-lift error order (the dates' #NUM!
before the unit's error); the inferred order is UNVERIFIED (T-1427). Remaining probe cells are the date arguments of
EDATE, EOMONTH, WEEKNUM, WORKDAY(.INTL), NETWORKDAYS(.INTL), YEARFRAC over an array (T-1236) and G3/G4 (T-1237).
Review notes: a single-cell range `weekend` (`NETWORKDAYS.INTL(1,30,C1:C1)`) and a 3D range in the lifted text/weekend
arguments are unprobed and keep the old behavior (`#VALUE!`; a 3D-range weekend is guarded so it never lifts over its first sheet); probe them before changing. Follow-ups:
T-1295 (WORKDAY.INTL week stepping), T-1428 (3D ranges in the lifted arguments, DATEDIF's unit and DAYS360's method
over an array, now `#N/IMPL!`).
