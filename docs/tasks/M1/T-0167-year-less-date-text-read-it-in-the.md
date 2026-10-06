---
id: T-0167
title: "Year-less date text: read it in the current year (engine clock access)"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`DATEVALUE("Jul-26")`, `VALUE("7/1")`, `DATEVALUE("1-Jul")` and the other year-less forms give the date in the current year, as Excel does.

## Context
Split from T-0160, which recorded the evidence (docs/excel-semantics/dates.md "Year-less date text",
probes/date-text-yearless) and implemented the clock-free half: a number that can never be a day of
the month is a two-digit year ("Jul-45" is July 1945). The current-year half needs the clock:
`formatter::date_text::parse_date` (called from VALUE/DATEVALUE, number coercion in `format.rs`,
criteria, conditional formatting, `time_text`) only gets a `Locale`. Design an engine-level "today"
(the same source as NOW/TODAY: `get_milliseconds_since_epoch` plus the model timezone) passed in, not
read by the parser (CLAUDE.md directive 5). In the current year, the number is a day when it is a valid
day of that month *this* year: in 2026 "2/29" and "Feb-29" are 1 Feb 2029, in a leap year 29 Feb. In
Excel these cells are not volatile; the harness scores only clock-independent cells
(probes/date-text-yearless B1:D12 MONTH/DAY, which fail today with #VALUE!).

## Acceptance criteria
- [ ] probes/date-text-yearless B1:D12 pass; baseline shrinks.
- [ ] The `date_text.rs` test that pins these forms to #VALUE! (`month_and_two_digit_year_when_the_number_cannot_be_a_day`) flips to the Excel values.
- [x] Probe the forms T-0160 left unprobed: `"7/0"`, `"7-0"`, `"1-0"`, `"13/45"`, `"0/45"`, `"7 45"` (space between numbers), `"7/2026"`, `"7-2026"`, `"12/1999"`, `"7/45 10:00"`, `"Jul 45"`, `"Jul/45"`, `"July 45"`, `"7-45"`, `"45-Jul"`, `"45 Jul"`; align the engine.
- [ ] Unit tests with a pinned clock: "Jul-26", "7/1", "1-Jul", "July 1", "Jul/26", "7-1", "7/1 10:00"; "2/29" in a leap and a non-leap year.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US; the display format typed year-less text gets.

## Progress (2026-10-06, opus run)
Done on this branch: probe `probes/date-text-yearless-2` (16 forms, excel-graph, 64 cells, all
clock-independent; none of them is read in the current year) recorded in dates.md, and the one
engine gap it showed fixed (numeric month with a four-digit year, `"7/2026"`, in
`parse_month_year`); the case passes 64/64. Left: the current-year half (pass an engine "today"
into `parse_date` and its callers: `format.rs` `parse_formatted_number`, `criteria.rs`,
`time_text::parse_date_clock_serial`, `date_and_time.rs` DATEVALUE), the pinned-clock unit tests,
and flipping the `Jul-26`-style rows of `month_and_two_digit_year_when_the_number_cannot_be_a_day`.
