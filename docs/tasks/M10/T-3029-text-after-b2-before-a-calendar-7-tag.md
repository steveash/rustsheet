---
id: T-3029
title: "TEXT: after B2 before a calendar-7 tag a weekday after mmm reads the month number"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT and cell display give Excel's weekday names after a `mmm` when `B2` comes before a calendar tag that wins over it
(calendar 7, and possibly others).

## Context
Found by T-2932. probes/t2932-b2-calendar-twice A55 `TEXT(45000,"B2[$-70409]yyyy-mm-dd|mmm|ddd")` is
"3109-03-15|มี.ค.|อ." (excel-graph 2026-10-28): `ddd` of 2023-03-15, a Wednesday, prints Tuesday ("อ."), weekday number
3 = the month number, as the Arabic `B2` path's first-name quirk does ("weekday number min(month, 7)",
`date_names::NameTokens::hijri_name`), but with the Thai names. Ours prints the real weekday "พ.".
So the first-name quirk is not only the Arabic path's. A55 is the only cell: probe before generalizing.
- `B2[$-70409]mmm|ddd`, `mmmm|dddd`, `dddd|mmm|dddd`, `yyyy|mmmm|dddd` (does `mmmm` trigger it, or only `mmm`?),
  across months 1-12;
- the same under calendars 1, 2, 3, 5, 9, A and a lunar digit after `B2` (`B2[$-10409]mmm|ddd`, `B2[$-30411]mmm|ddd`,
  `B2[$-50412]mmm|ddd`), and with no `B2` (`[$-70409]mmm|ddd`) as the control.
- T-2932's A54 `B2[$-30411]yyyy-mm-dd|mmmm|dddd` is the real weekday, so `mmmm` alone may not trigger it.

## Acceptance criteria
- [ ] probes/t2932-b2-calendar-twice A55 and the new probe's cells pass, with no new known failure.
- [ ] number-formats.md records the rule with the new probe's evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The years after `B2` (T-2932, T-3028).
