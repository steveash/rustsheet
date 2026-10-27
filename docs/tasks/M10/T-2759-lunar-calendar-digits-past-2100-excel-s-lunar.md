---
id: T-2759
title: "Lunar calendar digits past 2100: Excel's lunar dates to 9999"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Lunar calendar digits E, F, 11, 12 and 13 print Excel's lunar date for serials past 2100-12-31 (to 9999-12-31): `TEXT(2958465,"[$-110409]yyyy|m|d")` is "9999|11|26" (probes/text-locale-lunar-edges A20).

## Context
T-1922 tabulated Excel's two lunar month tables (digits F/11 and E/12/13) from 1900 to 2100 out of probes/text-locale-lunar-months-1900 and falls back to the Gregorian date past them (formatter/lunar_calendar.rs `LAST_DAY`). Outside lunar new year 1960 to 2051 Excel's months are not the published calendar (1-, 15- and 28-day months, 14-month years), so the rest is either a crack of Excel's algorithm (try a mean-lunation or new-moon/solar-term model against the 1900-1960 and 2051-2100 data first) or the same per-year probe extended to 9999 (about 8000 years, two columns; mind the probe file size, about 200 KB per 1000 years). See docs/excel-semantics/number-formats.md "Calendars of a locale tag".

Exit owners (decision 0096; the 2026-10-27 planner run names each cell the ranges above hold):
- probes/text-locale-lunar-edges S!A15, S!A16, S!A17, S!A18, S!A19

## Acceptance criteria
- [ ] probes/text-locale-lunar-edges A14-A20 and text-locale-lunar-tokens A80, A81 pass, plus new probes past 2100 for both tables (with a `yyyy` check at each month-1 start, to confirm the year turns only there).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Progress (2026-10-27, opus run; the task stays in progress on this branch)
- New probe probes/t2759-lunar-months-2101 (excel-graph, 2026-10-27): the per-year month-start strings of
  text-locale-lunar-months-1900 for 2101-2300 and 9990-9999, column A digit 11 (Chinese table), column B digit E
  (Eastern table). Not yet in eval/baseline: run `make eval-baseline` before the PR (all 420 cells fail today; the
  engine prints the Gregorian date past 2100).
- What it shows: past 2100 the lunar year number follows the Gregorian year closely (2101-01-01 is "2100/12/17",
  9999-01-01 "9998/12/16"), and the 1900-1960 artefacts return: 1-day months at the year turn (2105: month 13 on
  Jan 30, month 1 on Jan 31), 14-month years (2109: month 14 on Feb 15 lasts one day), 28-, 15- and 31-day months.
  Month lengths over 2101-2300: 30 x1283, 29 x1168, 1 x47, 28 x20, 15 x2, 31 x1. Columns A and B differ by a day at
  some month starts (2103 month 6, 2110 month 11), as in 1900-2100.
- Not true new moons: against Meeus' new-moon formula only 326 of 2522 starts match the local (UTC+8) day; at 2101 the
  starts sit about half a lunation off the new moon, and in 9990-9999 about 4-5.5 days after the mean new moon, so
  Excel's month cycle drifts against the real one. A straight mean-lunation fit over 2101-2300 (29.5337 days) leaves
  residuals of -12 to +20 days once months are indexed by rounding, so the indexing (or the model) is wrong.
- Next: index months by counting starts (not by rounding), fit day = floor(a + n*L) per table over 2101-2300, and
  test it on 9990-9999 and on 1900-1960 (lunar_calendar.rs tables). The year turn likely comes from a separate
  new-year rule (month 1 forced on a date that need not be a month start, hence the 1-day months); recover it from
  the forced starts. If no closed form appears within the time box, fall back to the per-year probe to 9999
  (about 8000 years x 2 columns), stored as packed binary data rather than Rust source (1000-line file limit).
