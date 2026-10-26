---
id: T-2759
title: "Lunar calendar digits past 2100: Excel's lunar dates to 9999"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Lunar calendar digits E, F, 11, 12 and 13 print Excel's lunar date for serials past 2100-12-31 (to 9999-12-31): `TEXT(2958465,"[$-110409]yyyy|m|d")` is "9999|11|26" (probes/text-locale-lunar-edges A20).

## Context
T-1922 tabulated Excel's two lunar month tables (digits F/11 and E/12/13) from 1900 to 2100 out of probes/text-locale-lunar-months-1900 and falls back to the Gregorian date past them (formatter/lunar_calendar.rs `LAST_DAY`). Outside lunar new year 1960 to 2051 Excel's months are not the published calendar (1-, 15- and 28-day months, 14-month years), so the rest is either a crack of Excel's algorithm (try a mean-lunation or new-moon/solar-term model against the 1900-1960 and 2051-2100 data first) or the same per-year probe extended to 9999 (about 8000 years, two columns; mind the probe file size, about 200 KB per 1000 years). See docs/excel-semantics/number-formats.md "Calendars of a locale tag".

## Acceptance criteria
- [ ] probes/text-locale-lunar-edges A14-A20 and text-locale-lunar-tokens A80, A81 pass, plus new probes past 2100 for both tables (with a `yyyy` check at each month-1 start, to confirm the year turns only there).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
