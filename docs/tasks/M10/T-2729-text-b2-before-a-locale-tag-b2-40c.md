---
id: T-2729
title: "TEXT: B2 before a locale tag (B2[40C]mmmm is Chaaban) and the [6xxxx] Hijri calendar digit"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT gives Excel's Hijri dates and names when `B2` comes before a locale tag and under a `[$-6xxxx]` calendar-6 tag, instead of `#VALUE!` or the Gregorian date.

## Context
T-1613 implemented the Hijri calendar of a bare `B2` prefix (formatter/date_names.rs, `hijri_ymd`, Arabic names).
Two forms stay unmodelled (number-formats.md "Calendars of a locale tag" and "`b` and `B1`/`B2` under a tag"):
- `B2` before a tag still selects Hijri, with the tag's names: `B2[$-40C]mmmm` is "Chaaban". The engine gives `#VALUE!`
  (formatter/parser.rs, the `tagged` check next to `TODO(T-2729)`).
- Calendar digit 6 is Hijri: `[$-60409]B2yyyy` is "1444", "1444-08-23", "Sha'ban". The engine prints the Gregorian date.
Probe the French, English and Arabic month and weekday names under each form first (excel-probe skill), and the name-state
quirk of probes/t1613-hijri-b2-weekday under a tag. In the same probe, pin the name-state combinations T-1613 left
unprobed (number-formats.md "The `B2` (Hijri) calendar"): `B2dddd|mmmmm`, `B2dddd|mmmm|dddd`, `B2mmmm|dddd|mmmm`,
`B2mmmm|dddd|dddd`, `B2mmmmm|mmmm`, and a `B2yyyy-mm-dd` cell number format (Range.text) next to TEXT.

Exit owners (decision 0096; the 2026-10-27 planner run names each cell this task's two forms cover):
- probes/text-locale-calendars-detail S!A7 (`B2[$-411]mmmm|e|g`), S!A8 (`B2[$-40C]mmmm|yyyy`)
- probes/text-locale-era-gregorian S!A21 (`[$-60409]B2yyyy`, Excel "1444", ours "2023")

## Acceptance criteria
- [x] A probe case covers both forms (excel-graph) and passes; number-formats.md records the rule.
- [x] The three exit-owner cells above pass and leave eval/baseline/probes.txt, with no new known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other calendar digits (8 Hebrew, 10 Saka, the lunar ones).

## Outcome
Two probes (excel-graph, 2026-10-27): probes/t2729-hijri-locale-tag (278 cells) and t2729-hijri-locale-tag-names (320).
Calendar digit 6 is `Calendar::Hijri` with each LCID's Hijri names (Arabic, French, Turkish, else English
transliterations; new tables in locale/excel_date_names.rs). A `B2` before a tag makes the lexer resolve the tag with
digit 0 read as 6 (`locale_tag::resolve_after_hijri_prefix`); any other digit wins over the `B2`. Under a `B2` before a
tag, `mmm` switches the section to the bare-`B2` Arabic path; in that path `mmm` resets the first-name value, `mmmmm`
after a weekday prints the whole weekday, and a number token before the first name turns the quirk off. A second calendar
prefix after a tag is `#VALUE!`, and `B1`/`B2` no longer conflict with `@` in TEXT. number-formats.md has a new section,
"`B2` before a locale tag and calendar digit 6".
Eval: probes known failures 2990 -> 2987 (10 fixed: text-locale-calendars A47-A49, A87, A88, A124; text-locale-calendars-detail A7, A8;
text-locale-era-gregorian A21; text-locale-lunar-tokens A79), 595 new probe cells of which 7 fail (owned by T-2932), 0 regressions.
Follow-up: T-2932 (`B2` before calendar 3 and 7 applies the year conversion twice; also lists the Hebrew `B2[$-80409]`
cell, which T-2779 fixes). T-2764's last cell (text-locale-lunar-tokens A79) passes now.
