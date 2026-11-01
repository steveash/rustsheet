---
id: T-2935
title: "Probe calendar digits 3, 4, 5, 9 and 14 over an LCID with no locale"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's output for calendar digits 3, 4, 5, 9 and 14 over an LCID with no locale (LCID 0 and, say, 0x8DE) is probed and
recorded; the engine matches it.

## Context
Left open by T-2764 (its Context asked to probe these digits before generalizing) and by T-1924. number-formats.md
"Calendars of a locale tag", the bullet on calendar digits over an LCID with no locale, lists 3, 4, 5, 9 and 14 as
unprobed; `without_locale` (formatter/locale_tag.rs) leaves them `LocaleTag::Silent`, so they print en-US Gregorian.
Probe (excel-probe skill, excel-graph) `TEXT(45000,"[$-30000]g|e|yyyy|mmmm|dddd|h AM/PM|A/P")` and the same for
`[$-40000]`, `[$-50000]`, `[$-90000]`, `[$-140000]` and the `08DE` LCID, with 45000.75 for the day halves.

## Acceptance criteria
- [ ] A probe case covers the five digits over LCID 0 and 0x8DE (excel-graph) and passes; number-formats.md records the
      rule in place of the "unprobed" note.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
