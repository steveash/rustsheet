---
id: T-1353
title: "HOUR, MINUTE and SECOND of a serial past 9999-12-31 are #NUM!"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
HOUR, MINUTE and SECOND of a serial past the last date (9999-12-31, serial 2958465) are `#NUM!`, as in Excel.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11); docs/excel-semantics/dates.md "HOUR, MINUTE and SECOND past 9999-12-31":
- gen-m7/bf889c3917eb54c5 Calc!A1 `=HOUR(123456789.123)`: Excel `#NUM!`, engine 2.
- gen-m7/851eb2099ae3f805 Calc!A39 `=(COUNT(Data!D9)>=MINUTE((7+123456789.123)))`: Excel `#NUM!`, engine FALSE.
`time_part_fn!` (crates/rustsheet-engine/src/functions/date_and_time.rs) only rejects negative values; DAY already rejects
2958466 using `MAXIMUM_DATE_SERIAL_NUMBER`. Use the same constant. The exact bound (does 2958465.99999 still give a time,
is 2958466 `#NUM!`) is not evidenced: probe `HOUR(2958465.9)`, `HOUR(2958466)`, `MINUTE(2958466)`, `SECOND(2958466)` and
`SECOND(1E+15)` with the excel-probe skill before choosing `>` or `>=`.

## Acceptance criteria
- [ ] gen-m7/bf889c3917eb54c5 A1 and gen-m7/851eb2099ae3f805 A39 pass in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] The bound probe cells pass; dates.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed the bound (probes/time-parts-past-max-date, 14 cells): HOUR, MINUTE and SECOND are `#NUM!` from serial 2958466; `time_part_fn!` now rejects `>= MAXIMUM_DATE_SERIAL_NUMBER + 1`. dates.md is VERIFIED. gen-m7/bf889c3917eb54c5 A1 and gen-m7/851eb2099ae3f805 A39 now pass.
