---
id: T-3101
title: "TEXT: B2 before calendar 1, 3 or 5 tags changes the names' language"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT and cell display print Excel's month and weekday names when a `B2` comes before a calendar-1, -3 or -5 tag, whose names Excel then takes from another language.

## Context
Found by T-3029 (number-formats.md "`B2` before a locale tag and calendar digit 6", the T-3101 rule).
probes/t3029-b2-calendar-weekday A73-A75 `B2[$-10409]mmm|ddd` are Arabic Gregorian names ("مارس|الثلاثاء") where
`[$-10409]` alone is English; A79-A84 `B2[$-30411]mmm|ddd` and `B2[$-50412]mmm|ddd` (and B3 as a cell format) are English
("Mar|Tue") where the tags alone print Japanese and Korean names. `B2[$-30409]`, `B2[$-50409]`, `B2[$-7041E]`,
`B2[$-90409]` and `B2[$-A0409]` keep the tag's names. The weekday numbers (min(month, 7) after `mmm`) are already right.
Probe before generalizing: `B2[$-10411]`, `B2[$-1040C]`, `B2[$-10401]`, `B2[$-3040C]`, `B2[$-5040C]`, `B2[$-20411]`,
`B2[$-30412]`, `mmmm`/`dddd`/`mmmmm` and a `mmm`-free section (`B2[$-30411]dddd`, `B2[$-10409]mmmm`).
Code: `resolve_after_hijri_prefix` (formatter/locale_tag.rs), `NameTokens::push` (formatter/date_names.rs).

## Acceptance criteria
- [ ] probes/t3029-b2-calendar-weekday A73-A75, A79-A84 and B3 pass, with the new probe's cells, and no new known failure.
- [ ] number-formats.md records the rule with the new evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The weekday after `mmm` (T-3029).
