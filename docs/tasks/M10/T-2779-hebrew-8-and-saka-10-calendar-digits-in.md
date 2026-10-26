---
id: T-2779
title: "Hebrew (8) and Saka (10) calendar digits in locale tags print Excel's dates and names (probes/t1923-calendar-digits)"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Calendar digits 8 (Hebrew) and 10 (Saka) in a `[$-...]` tag print the date, month and weekday names Excel prints.

## Context
Split from T-1923, which modelled digits A, B, C and 14. The rules and evidence are in number-formats.md "Calendars of a locale
tag" ("Hebrew (8)" and "Saka (10)"), from probes/t1923-calendar-digits (excel-graph, 2026-10-26): A85:A144 and A205/A207/A209
for Saka, A145:A204, A206/A208/A210 and A211:A213 for Hebrew. Older cells: probes/text-locale-calendars A53:A55, A77:A79,
A89, A90 and their neighbours.
Code: formatter/locale_tag.rs `resolve_lcid` (both digits are `LocaleTag::Silent`, `TODO(T-2779)`). Add a `Calendar` variant per
calendar, convert like the lunar calendar's date tokens, and add the name tables to locale/excel_date_names.rs.
- Saka: the year turns on 22 March (21 March in a Gregorian leap year: 2024-03-21 is 1946-01-01).
  - Chaitra has 31 days in a leap year.
  - `yy` prints two digits; `e` and `b` print the Saka year.
  - `ddd` is a short Sanskrit weekday, Sunday "Ravi" to Saturday "Sani" (the seven in A85:A144). `dddd` is A77's "Budhavara" form, which needs its own probe.
- Hebrew: the full arithmetic calendar. Months number from Tishrei = 1, so a leap year has 13: AdarI is 6, AdarII 7, Elul 13.
  - Month names are Latin under non-Hebrew LCIDs and Hebrew script under 40D and 401 ("אדר ב").
  - Weekdays are "Yom Rishon" … "Shabat".
  - 9999-12-31 is `#VALUE!`. Serials 0 and 1 are 5660-05-01 (Shvat), and serial 60 is `#VALUE!` under both calendars.
Hebrew-script weekdays join with a no-break space ("יום רביעי", A211); "אדר ב" uses a plain space (A212). Also probe `yy|b|bbbb`
under digits A, B, C and 14 (T-1923 inferred them as Gregorian). Probe first: Saka `dddd` for every weekday, Hebrew `ddd`, and the Hebrew names under 40D for all 14 month forms.

## Acceptance criteria
- [ ] The Saka and Hebrew cells of probes/t1923-calendar-digits and probes/text-locale-calendars leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
