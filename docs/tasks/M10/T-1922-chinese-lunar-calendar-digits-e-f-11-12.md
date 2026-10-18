---
id: T-1922
title: "Chinese lunar calendar digits E, F, 11, 12 and 13 in locale tags"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Calendar digits E, F, 11, 12 and 13 in a `[$-...]` tag print the Chinese lunar date as Excel does: `TEXT(45000,"[$-11040C]mmmm")` is "二月" and `[$-11040C]d` "24".

## Context
T-1859 probed them (probes/text-locale-calendars A71-A85, A98-A115; text-locale-calendars-detail A30) and left them `Silent`. All five give the lunar month and day (45000 = 2023-02-24 lunar). `yyyy` and `e` stay Gregorian (2023). The names differ: E is Chinese months with Japanese days (水曜日), F and 11 Chinese months and days (星期三), 12 and 13 English names ("February"). The first lunar month is "正月". Leap months are unprobed: probe 2023's leap 2nd month (from 2023-03-22) and years near 1900 and 2100. A lunar table needs a source that matches Excel over 1900-9999, or a documented range limit. See docs/excel-semantics/number-formats.md "Calendars of a locale tag". Code: formatter/locale_tag.rs `resolve_lcid`, `Calendar`; format.rs.

## Acceptance criteria
- [ ] probes/text-locale-name-tables A589 and the lunar cells of text-locale-calendars pass, and leap-month probes are recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
