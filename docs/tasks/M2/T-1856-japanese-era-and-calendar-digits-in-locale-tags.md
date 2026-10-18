---
id: T-1856
title: "Japanese era and calendar digits in locale tags"
milestone: M2
lane: functions-text
status: todo
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
- [ ] The six cells above pass; the new probes are recorded in number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
