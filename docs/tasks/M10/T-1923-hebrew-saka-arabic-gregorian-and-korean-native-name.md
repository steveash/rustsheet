---
id: T-1923
title: "Hebrew, Saka, Arabic Gregorian and Korean native-name calendar digits in locale tags"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Calendar digits 8 (Hebrew), 10 (Saka), A, B and C (Arabic Gregorian) and 14 (Korean native names) in a `[$-...]` tag print what Excel prints.

## Context
T-1859 probed them and left them `Silent` (probes/text-locale-calendars A53-A55, A59-A67, A77-A79, A89-A90; text-locale-calendars-detail A31). The values are:
- 8: Hebrew date "5783-06-22" with "Adar" and "Yom Revi'i", or Hebrew script under 40D and 401.
- 10: Saka date "1944-12-24" with "Phalguna" and "Budhavara".
- A: Levant Arabic month names ("آذار").
- B and C: "مارس".
- 14: Gregorian with Korean native month names ("삼월").

Hijri (6) is T-1613's. See docs/excel-semantics/number-formats.md "Calendars of a locale tag". Code: formatter/locale_tag.rs `resolve_lcid`.

## Acceptance criteria
- [ ] Those cells pass; more dates are probed for the Hebrew and Saka year boundaries.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
