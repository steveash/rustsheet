---
id: T-3068
title: "Hebrew calendar: probe the month-name limit under other locales and the second year-table wrap (2892-2893)"
milestone: M10
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that T-2954's Hebrew-calendar model holds where it is extrapolated, or a fix where it does not.

## Context
T-2954 modelled Excel's Hebrew dates as a year table that wraps past 2238: Gregorian year y reads the entry of 1585 + (y - 1585) mod 654. Month names under a locale are `#VALUE!` from 1 Tishrei 8263 (4502-09-30). Both rules were probed only with `[$-80409]` (and `[$-80000]`). Unprobed:
- the limit under `[$-8040D]` (Hebrew-script names), `[$-80407]` and `[$-80401]`, at 4502-09-29 and 4502-09-30, with `mmm`, `mmmm` and `mmmmm`;
- the second wrap boundary: `yyyy-mm-dd` of the 1st of each month in 2892-2893, plus 2892-12-31 and 2893-01-01.
Code: formatter/calendar_dates.rs (`hebrew_table_entry`, `HEBREW_LAST_NAMED_YEAR`) and formatter/format.rs. Docs: number-formats.md "Hebrew (8)".

## Acceptance criteria
- [ ] A probe case with these cells is oracle-filled (excel-graph) and passes, or each mismatch is fixed here or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other calendars.
