---
id: T-3176
title: "YEAR, MONTH, DAY and WEEKDAY take HOUR's half-second carry past midnight, through one helper with time_of_day_parts"
milestone: M2
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
YEAR, MONTH, DAY and WEEKDAY read a serial within half a second of the next midnight as the next day, exactly where
HOUR, MINUTE and SECOND roll to 0, and every date and time part is `#NUM!` from `fl(2958466-0.5/86400)` up. All 223
failing cells of probes/t3137-date-parts-near-midnight pass.

## Context
- Rule and evidence: dates.md "YEAR, MONTH and DAY of a serial just below midnight" and the T-3137 paragraph of
  "HOUR, MINUTE and SECOND past 9999-12-31" (probes/t3137-date-parts-near-midnight, excel-graph 2026-10-29). The day
  is `floor(x + 0.5/86400)` in double precision. A negative serial is `#NUM!` before any carry, and so is a day past
  2958465.
- Failing cells (baseline known failures in eval/baseline/probes.txt): R!B, C, D and E (YEAR, MONTH, DAY, WEEKDAY)
  of the carried rows. Those are rows 5-14, 20-29, 35-44, 50-59, 65-74, 80-89, 95-104 and 110-119, wherever the
  part changes across midnight. R!F110, G110 and H110 are also failing: HOUR, MINUTE and SECOND at the exact tie
  2958465.9999942128 are `#NUM!` in Excel and 0 in ours.
- Code: `Model::date_part` (crates/rustsheet-engine/src/functions/date_and_time.rs) floors the serial. WEEKDAY's own
  serial read needs the same carry. `Model::time_part` bounds the serial with `days360_serial(value) <= last`, which
  misses the exact tie. `time_of_day_parts` adds `0.5 / SECONDS_PER_DAY_F64` already. Add one helper for the
  carried day (`floor(v + 0.5/86400)`) and use it in date_part, WEEKDAY and time_part's bound. Check that
  days360_serial's other callers keep their behavior. Do not write a second rounding.

## Acceptance criteria
- [x] Every cell of probes/t3137-date-parts-near-midnight passes. `make eval-baseline` shrinks the known failures, with
      0 regressions in every suite (probes/second-rounding, time-parts-rounding-window, time-parts-past-max-date,
      weekday-serial-zero included).
- [x] A unit test pins `DAY(1-0.5/86400)` = 1, one ULP below it = 0, and `YEAR(2958466-0.5/86400)` = `#NUM!`.
- [x] dates.md's "Ours" line says the engine matches.
- [x] `make check` green; reviewers approve.

## Out of scope
- EDATE, EOMONTH, DATEDIF, YEARFRAC, WORKDAY and the other date readers (unprobed at the carry).
- The 1904 date system.

## Outcome
Added `carried_day` in `date_and_time.rs`, used by `date_part`, WEEKDAY and `time_part`'s bound (explicit
`> 2958465` check in `date_part`). Unit test `date_parts_carry_past_midnight`. Eval: 223 known failures fixed
(probes/t3137-date-parts-near-midnight), 0 regressions.
