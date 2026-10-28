---
id: T-3028
title: "TEXT: B2 before a calendar-4 (Taiwan) tag prints 01 years"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT and cell display give Excel's years when `B2` comes before a calendar-4 (Taiwan) locale tag.

## Context
Found by T-2932 (number-formats.md "`B2` before a locale tag and calendar digit 6"). Before calendars 3, 5 and 7 a `B2`
converts every year token once more (implemented by T-2932), but before calendar 4 Excel prints neither the Gregorian
nor a twice-converted year:
- probes/t2932-b2-calendar-twice A37 `TEXT(45000,"B2[$-40409]yyyy|yy|e|b|bbbb|g")` is "01|01|01|2|01|" and A38
  `B2[$-40404]yyyy|yy|e|b|bbbb|mmmm` "01|01|01|2|01|三月" (excel-graph 2026-10-28); ours prints the Gregorian years.
A guess to test: Taiwan's year is the Minguo year (2023 - 1911 = 112), read again as a Gregorian year (112 - 1911 is
before year 1), clamped to year 1; `b` "2" (one digit) is unexplained. Probe other serials (1900-01-01, 1912-01-01,
9999-12-31, a year whose Minguo year is past 1911, if any) and each year token alone (`B2[$-40409]b`, `bb`, `bbbb`, `e`,
`ee`, `yyy`) before implementing.

## Acceptance criteria
- [ ] probes/t2932-b2-calendar-twice A37 and A38 pass and leave eval/baseline/probes.txt, with no new known failure.
- [ ] number-formats.md records the rule with the new probe's evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calendars 3, 5 and 7 after `B2` (T-2932).
