---
id: T-1428
title: "Probe 3D ranges in lifted date text and weekend arguments, and DATEDIF unit and DAYS360 method over an array"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The unprobed argument modes left by T-1235 are probed and ours matches Excel: a multi-cell 3D range in the lifted
text/weekend arguments, and DATEDIF's unit and DAYS360's method over an array.

## Context
T-1235 review notes. Ours today: a 3D range (`Sheet1:Sheet2!A1:A3`) in DATEVALUE/TIMEVALUE's text or the
`weekend` of WORKDAY.INTL/NETWORKDAYS.INTL reads the first sheet only (or errors); DATEDIF's unit
(`DATEDIF(1,100,{"d","m"})`) and DAYS360's method (`DAYS360(1,100,{TRUE,FALSE})`) over an array are `#N/IMPL!`,
read through the scalar `get_string`/`get_boolean`.
Semantics: docs/excel-semantics/dates.md "Date function argument modes" and "DATEDIF and DAYS360 error order".
Code: `fn_datevalue`, `fn_timevalue`, `weekend_arg`, `fn_datedif`, `fn_days360` in
crates/rustsheet-engine/src/functions/date_and_time.rs. Use the `excel-probe` skill first.

Update (T-1785): DATEDIF's unit and DAYS360's method are probed (probes/date-math-lift-driver-edges G1-G10, G40-G41: they lift over arrays and ranges) and implemented as `Lift` entries of the SPEC-0011 driver; a 3-D reference in DAY, HOUR, DATE, WEEKDAY, DATEDIF and ISOWEEKNUM is `#REF!` there (G24-G26, G30-G32). What is left here is the 3D range in DATEVALUE/TIMEVALUE's text and in the `weekend` argument.

## Acceptance criteria
- [x] A probe case covers the four modes above with oracle values; ours matches them (baseline updated) or a
  follow-up implements the difference.
- [x] dates.md records the results.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New probe probes/date-text-weekend-3d (125 cells, excel-graph 2026-10-19), recorded in dates.md "An error-valued or 3-D
weekend, 3-D date text and holidays". A 3-D reference in DATEVALUE/TIMEVALUE's text is `#REF!` (ours already matched).
The `weekend` differed: WORKDAY.INTL turns any error-valued weekend (a 3-D one reads as `#REF!`) into `#NUM!` and reads
it after the start, day count and holidays; NETWORKDAYS.INTL reads the weekend first and turns an error-valued one into
`#VALUE!`. A 3-D `holidays` is `#VALUE!`. `weekend_mask` takes the caller's error code, `workday_of`/`networkdays_of`
reorder their reads, and `holiday_serials` refuses a 3-D reference. Eval delta: +125 probe cells, all passing (the 12
weekend and holidays cells that failed before the fix now pass); no other cell changed.
