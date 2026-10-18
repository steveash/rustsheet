---
id: T-1859
title: "Japanese era and calendar digits in locale tags"
milestone: M2
lane: functions-text
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Era codes and calendar digits in a locale tag print what Excel prints: `[$-411]g|gg|ggg|e|ee` of 45000 is "R|令|令和|5|05", `[$-11040C]mmmm` "二月", `[$-40C]bbbb` "2023" and `[$-40C]B2mmmm` "mars".

## Context
T-1716 localized names under `[$-LCID]` tags but left eras and calendar digits (the hex digits above the low four) unmodelled; `locale_tag::resolve` returns `Silent` for any calendar digit but 0 and 1. Failing cells: probes/text-locale-name-tables A105, A546 (Japanese era), A587 (B2 under 40C), A588 (bbbb under 40C), A589 (calendar 11), A596 (`[$-fr_FR]` gives "December"). Probe more eras (Heisei/Showa dates, `[$-411]ggge"年"`), other calendar digits (02-12, `[$-1010409]`?) and Buddhist under other LCIDs before implementing. See docs/excel-semantics/number-formats.md "Locale tags localize date names". Code: formatter/locale_tag.rs, format.rs (`TextToken::Era`, Buddhist years).

## Acceptance criteria
- [x] A105, A546 (Japanese era), A587 (B2 under 40C) and A588 (bbbb under 40C) pass, and the new probes are recorded in number-formats.md.
- [x] A589 (calendar 11 is the Chinese lunar date) moved to T-1922. A596 (`[$-fr_FR]` selects the Saka calendar) moved to T-1924. The probes showed both are calendar arithmetic, not name tables, and too big for this task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Three new probes landed: probes/text-locale-calendars (145 cells), text-locale-calendars-detail (41) and text-locale-japanese-year (20). number-formats.md "Calendars of a locale tag" records them.
- Locale tags now carry a calendar (`locale_tag::Calendar`). This adds Japanese eras: `g`/`gg`/`ggg`, `e`/`ee`, and `yy`/`yyyy` under calendar 3, or after a `g` under a Japanese locale.
- Calendar digits 2, 4, 5, 7, 9, D and 15+ are modelled.
- `b` under a tag is the Gregorian year, except under Thai.
- `B1`/`B2` after a tag are ignored.
- Day halves follow the LCID.

Eval: name-tables A105, A546, A587 and A588 fixed; 139 of the 206 new cells pass. The other 67 are new-coverage failures, filed by calendar:
- T-1922: lunar
- T-1923: Hebrew, Saka, Arabic, Korean native names
- T-1924: underscore tags
- T-1925: numeral digits
- T-1613: Hijri
