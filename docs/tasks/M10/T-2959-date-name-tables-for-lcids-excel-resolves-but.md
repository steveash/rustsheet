---
id: T-2959
title: "Date-name tables for LCIDs Excel resolves but we have no table for (bn-IN 445 prints the Latin Saka names)"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: sonnet
depends_on: []
---

## Goal
A `[$-...]` tag over an LCID Excel resolves but that has no name table here (bn-IN 0x445 first) prints Excel's names, not the no-locale en-US fallback.

## Context
`locale_tag.rs` treats an LCID with no table in `locale/excel_date_names.rs` `LOCALES` (and no language table) as having no
locale. Excel resolves more: `TEXT(45000,"[$-100445]mmm|mmmm|ddd|dddd")` is "Phalguna|Phalguna|Budh|Budhavara" (the
Latin Saka names), ours "Dec|December|Wed|Wednesday" (probes/t2779-hebrew-saka-names E18, T-2779). Probe 0x445's
Gregorian names (and other Indic LCIDs: 0x44E, 0x449, 0x44A, 0x44B, 0x457, 0x461) and add their tables, or a list of
LCIDs Excel resolves, so the calendar digits apply. number-formats.md "Hebrew (8) and Saka (10)" notes the miss.

## Acceptance criteria
- [ ] probes/t2779-hebrew-saka-names E18 leaves eval/baseline/probes.txt, with a probe of the new tables.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
