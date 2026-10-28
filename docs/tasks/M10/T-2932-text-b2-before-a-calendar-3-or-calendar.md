---
id: T-2932
title: "TEXT: B2 before a calendar-3 or calendar-7 tag applies the year conversion twice"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT and cell display give Excel's years when `B2` comes before a Thai (calendar 7) or explicit Japanese (calendar 3)
locale tag, where Excel applies the calendar's year conversion a second time.

## Context
Found by T-2729 (number-formats.md "`B2` before a locale tag and calendar digit 6", the calendar-3/7 quirk). Any calendar
digit but 0 and 6 wins over a `B2` before the tag, and the engine prints such a section as if the `B2` were not there.
Excel does so for digits 1, 2, 9, A and the lunar ones, but for 7 and 3 it converts the year twice:
- `B2[$-70409]yyyy-mm-dd|mmmm|dddd|g|e` of 45000 is "3109-03-15|มีนาคม|พุธ||2566" (2023 + 543 twice; `e` the Thai year once),
  `B2[$-70409]b|bbbb|e` "09|3109|2566".
- `B2[$-30411]yyyy-mm-dd|mmmm|dddd|g|e` is "4294965434-03-15|3月|水曜日|M|4294965434" and `B2[$-30409]yyyy|e|g`
  "4294965434|4294965434|M": the Reiwa era year 5 read again as a Gregorian year, which falls in Meiji (5 - 1868 + 1 =
  -1862, printed as an unsigned 32-bit number), and `g` that era's letter.
Probe before generalizing: `B2` before calendar 5 (Korean, +2333) and 4 (Taiwan), `B2` before `[$-411]` with a `g`
(`B2[$-411]g|yy`, the Japanese LCID's own calendar becomes Hijri there), other serials (an early Reiwa date, a Meiji
date), and `B2[$-70409]`/`B2[$-30411]` as a cell number format. In the same probe, confirm two inferences T-2729 left
(number-formats.md): `TEXT(45000,"B1@")` and `"B2@"` (inferred "45000" from `B2[$-409]@`), and a number before the
`mmm` switch, `B2[$-409]yyyy|mmm|dddd` (the engine prints the real weekday).
Failing cells (probes, excel-graph 2026-10-27):
- probes/t2729-hijri-locale-tag S!A245, A246
- probes/t2729-hijri-locale-tag-names S!A296, A297, A316, A317
Also failing, through T-2779 (Hebrew is not modelled; once it is, digit 8 wins over `B2` with no further change):
- probes/t2729-hijri-locale-tag S!A262 (`B2[$-80409]`, Excel "5783-06-22|Adar|Yom Revi'i||5783")

## Acceptance criteria
- [x] The six calendar-3/7 cells above pass and leave eval/baseline/probes.txt, with no new known failure.
- [x] number-formats.md records the rule with the new probe's evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- The Hebrew and Saka calendars themselves (T-2779).

## Outcome (2026-10-28)
Probe probes/t2932-b2-calendar-twice (73 cells, excel-graph): `B2` before a calendar-3, -5 (Korean, not only 3 and 7)
or -7 tag converts every year token once more; Japanese era years are read back as Gregorian years (negative ones
print as unsigned 32-bit numbers, 9999-12-31 is Reiwa 5963). Also found: `b` prints the year's third and fourth digits
(differs past 9999, with or without a tag), and under a tag a number token before the `mmm` switch leaves the
first-name quirk on. Implemented in `Localized::year_twice`, `locale_tag::japanese_era` and
`calendar_dates::year_token_text` (the year logic moved out of format.rs, at its size cap).
Eval: probes 2134 -> 2131 known failures (the six owned cells fixed; three new cells of the new case fail, owned by
follow-ups **T-3028** (B2 before calendar 4: A37, A38) and **T-3029** (Thai weekday after `mmm`: A55)); no regressions.
