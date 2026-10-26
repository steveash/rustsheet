---
id: T-1923
title: "Hebrew, Saka, Arabic Gregorian and Korean native-name calendar digits in locale tags"
milestone: M10
lane: functions-text
status: done
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

## Outcome
- Probe probes/t1923-calendar-digits (excel-graph, 2026-10-26, 213 cells): every month and weekday name of digits A, B, C
  and 14, their day halves, and Saka and Hebrew dates across 2023-2024 and at their year boundaries. Findings are in
  number-formats.md "Calendars of a locale tag".
- Digits A (Levant Arabic months, `ARABIC_LEVANT`), B and C (ar-SA names, `ARABIC`) and 14 (Korean native months,
  `LUNAR_KO`) now resolve as Gregorian calendars with those names (formatter/locale_tag.rs `resolve_lcid`).
- Split: Hebrew (8) and Saka (10) need their own date arithmetic and go to T-2779 (opus, M). This task's Hebrew and Saka
  acceptance cells (text-locale-calendars A53:A55, A77:A79, A89, A90) and the new probe's A85:A213 move there.
- Eval delta: probes fixed 10 (text-locale-calendars A59:A67, text-locale-calendars-detail A31), 0 regressions. The new
  case holds 129 known failures, all Saka and Hebrew (T-2779).
