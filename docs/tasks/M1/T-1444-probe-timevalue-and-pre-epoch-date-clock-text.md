---
id: T-1444
title: "Probe TIMEVALUE and pre-epoch date-clock text with an inexact clock in a 1904 workbook"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In a 1904 workbook, TIMEVALUE of date-clock text whose clock is inexact matches Excel to the last bit.

## Context
From the T-1441 fidelity review. `time_text.rs` TIMEVALUE fallback takes the fraction of the 1900-based `parse_date_clock_serial` sum; Excel may take it from the shifted 1904 sum, which differs in the low bits (1900 precedent: `TIMEVALUE("1/2/2020 25:00")` = 0.04166666666424135). Also unprobed: `"1903-12-31 25:00"+0` (pre-epoch day with an hour past 23).

## Acceptance criteria
- [x] A probe in a 1904 workbook with `TIMEVALUE("1/2/2020 25:00")`, `TIMEVALUE("1904-01-02 10:11:12.5")` and `"1903-12-31 25:00"+0`, Excel values from the oracle; the engine matches; dates.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `date-system-1904-timevalue` (7 cells, excel-graph). TIMEVALUE's date-clock fallback now shifts the date to the workbook's system before adding the clock (`parse_date_clock_serial_in`, shared with `parse_time_serial`): the fraction comes from the 1904 sum and a pre-epoch day is #VALUE!. 2 cells newly pass; dates.md updated.
