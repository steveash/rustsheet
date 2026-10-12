---
id: T-1441
title: "1904 workbooks: date-clock text coerced to a number, and a negative 1904 holiday serial in WORKDAY"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In a 1904 workbook, date-clock text (`"1904-01-02 10:00"+0`) coerces to its 1904 serial, and a negative 1904 holiday serial in WORKDAY matches Excel.

## Context
Split from T-1435, which shifted date text, typed dates and date criteria. `time_text::parse_date_clock_serial` (read by `parse_time_serial`) still counts from 1900; probe it first. `=WORKDAY(DATE(2024,3,15),1,-1)` is #NUM! in ours and needs a probe. See dates.md "The 1904 date system".

## Acceptance criteria
- [x] Probe cases for both with Excel values; the engine matches; dates.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `date-system-1904-clock-text` (13 cells, excel-graph). Date-clock text now reads the date part in the workbook's date system (`parse_time_serial` takes the `DateSystem`, shifts the day before adding the clock to keep Excel's low bits; a pre-epoch day is #VALUE!). The negative-holiday WORKDAY case was already right: Excel gives #NUM! too. Probes: 5 cells newly pass (no known-failure entries). dates.md updated.
