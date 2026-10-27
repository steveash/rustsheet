---
id: T-2759
title: "Lunar calendar digits past 2100: Excel's lunar dates to 9999"
milestone: M10
lane: functions-text
status: done
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
- [x] probes/text-locale-lunar-edges A14-A20 and text-locale-lunar-tokens A80, A81 pass, plus new probes past 2100 for both tables (with a `yyyy` check at each month-1 start, to confirm the year turns only there).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Progress (2026-10-27, opus runs; superseded by the Outcome below)
- New probe probes/t2759-lunar-months-2101 (excel-graph, 2026-10-27): the per-year month-start strings of
  text-locale-lunar-months-1900 for 2101-2300 and 9990-9999, column A digit 11 (Chinese table), column B digit E
  (Eastern table).
- What it shows: past 2100 the lunar year number follows the Gregorian year closely (2101-01-01 is "2100/12/17",
  9999-01-01 "9998/12/16"), and the 1900-1960 artefacts return: 1-day months at the year turn (2105: month 13 on
  Jan 30, month 1 on Jan 31), 14-month years (2109: month 14 on Feb 15 lasts one day), 28-, 15- and 31-day months.
  Month lengths over 2101-2300: 30 x1283, 29 x1168, 1 x47, 28 x20, 15 x2, 31 x1. Columns A and B differ by a day at
  some month starts (2103 month 6, 2110 month 11), as in 1900-2100.
- Not true new moons: against Meeus' new-moon formula only 326 of 2522 starts match the local (UTC+8) day; at 2101 the
  starts sit about half a lunation off the new moon, and in 9990-9999 about 4-5.5 days after the mean new moon, so
  Excel's month cycle drifts against the real one. A straight mean-lunation fit over 2101-2300 (29.5337 days) leaves
  residuals of -12 to +20 days once months are indexed by rounding, so the indexing (or the model) is wrong.
- Second pass (same day): leap months always come at the end of the year (months 13 and 14), because months are
  numbered from a separately computed new-year date: 2101-02-14, 2102-02-03, 2103-01-23, 2104-02-10, 2105-01-31,
  2106-02-19, 2107-02-08, 2108-01-28, 2109-02-16, 2110-02-05 (column A). Where that date is not a month start it cuts a
  1-day or 28-day stub. With those forced starts removed, the remaining month starts of column A fall into three
  segments with a mean lunation of 29.5294-29.5305 days and residuals of about +-1.9 days: 2101-01-15 to 2142-01-13,
  2142-02-27 to 2233-01-13, and 2233-02-27 on. The segments are separated by the two 15-day months, both ending on
  27 February (2142, 2233), where the phase jumps by about half a lunation. Script: count month indexes from the
  gaps (round(gap/29.53)), then fit each segment.
- Against Meeus' true new moon (UTC days, no delta-T): segment 2 (2142-2233) starts within -2..+3 days of the new
  moon (mostly 0 or +1), while segments 1 and 3 start 13-15 days after it, at mean phase 0.44-0.59, i.e. near the
  full moon. So Excel's lunation finder locks to the new moon or to the full moon and flips every ~91 years, at the
  27 February 15-day months, like an angle reduced modulo 180 degrees instead of 360. Its spread (+-2 days) is wider
  than the true moon's, so it is a cruder phase model than Meeus. Crack that model first (mean elongation plus a few
  periodic terms, evaluated at 0h of each day; find the flip condition), then the new-year rule.
- Next: index months by counting starts (not by rounding), fit day = floor(a + n*L) per table over 2101-2300, and
  test it on 9990-9999 and on 1900-1960 (lunar_calendar.rs tables). The year turn likely comes from a separate
  new-year rule (month 1 forced on a date that need not be a month start, hence the 1-day months); recover it from
  the forced starts. If no closed form appears within the time box, fall back to the per-year probe to 9999
  (about 8000 years x 2 columns), stored as packed binary data rather than Rust source (1000-line file limit).

- Resumed by an opus run at 2026-10-27 19:30 UTC (branch idle 2 h 45 min, work-loop resume rule).

## Outcome
- No closed form: Excel's month starts follow neither the true nor a mean new moon (progress notes above), so
  the fallback landed. probes/t2759-lunar-packed-2101 … -9101 (excel-graph, 2026-10-27; 8 cases, 158 cells, one
  per century and table) return one character per month start from 2101 to 9999 plus every day that breaks the
  counting rules; `scripts/lunar_tables.py pack` cross-checks each century's 1 January and writes
  formatter/lunar_2101.bin (200 KB: 99,157 month lengths per table and the early years), which
  lunar_calendar.rs decodes once and binary-searches.
- Quirks recorded (number-formats.md "Lunar calendars"): the 1960/2051 early year recurs every 91 years
  (2142, 2233, … 9968, 87 in all, both tables on the same days); when that month spans 31 days Excel repeats
  the day number on 1 January (2232-12-31 and 2233-01-01 are both "12/19"), and no other month has 31 days.
- Eval: probes/text-locale-lunar-edges A14-A20 and text-locale-lunar-tokens A80, A81 now pass (9 known failures
  fixed), t2759-lunar-months-2101 passes 420/420 and the 8 packed cases 158/158. The packed probe spells its codes
  with MID over a fixed alphabet rather than CHAR, which does not lift over arrays in RustSheet yet (follow-up T-2946).
