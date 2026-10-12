---
id: T-1444
title: "Probe TIMEVALUE and pre-epoch date-clock text with an inexact clock in a 1904 workbook"
milestone: M1
lane: functions-date
status: todo
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
- [ ] A probe in a 1904 workbook with `TIMEVALUE("1/2/2020 25:00")`, `TIMEVALUE("1904-01-02 10:11:12.5")` and `"1903-12-31 25:00"+0`, Excel values from the oracle; the engine matches; dates.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
