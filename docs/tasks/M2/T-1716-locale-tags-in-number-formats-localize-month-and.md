---
id: T-1716
title: "Locale tags in number formats localize month and day names"
milestone: M2
lane: functions-text
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A `[$-LCID]` tag in a number format localizes the date names and layouts it shows, as Excel does, instead of being a silent locale.

## Context
T-1714 probed `[$-40C]mmmm` on 45000 as "mars", `[$-407]mmmm` "März", `[$-40C]dddd` "mercredi" and `[$-F800]dddd` "Wednesday, March 15, 2023" (the system long date). The engine prints English names. Failing cells: probes/text-ap-locale-inferences A6:A9, probes/displayed-text-ap-locale A8. Code: formatter/lexer.rs `Token::Locale`, formatter/format.rs. See docs/excel-semantics/number-formats.md, "A/P letters and a locale-only [$-409]".

## Acceptance criteria
- [x] The five failing cells above pass; probe further tags (`[$-407]` days, `[$-40C]` AM/PM, `[$-F400]`) and record them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`[$-tag]` now localizes `mmm`/`mmmm`/`mmmmm`/`ddd`/`dddd` and the day-half strings from tables of Excel's own names for 25 LCIDs (formatter/locale_tag.rs), with LCID-language and BCP-47 fallbacks; `A/P` prints the locale's AM/PM under a resolved tag; `[$-F800]`/`[$-F400]` (and `x-sysdate`/`x-systime`) replace a first section with the system long date/time and empty a later one; a second tag in a section is `#VALUE!`. Probed 777 cells (probes/text-locale-date-names, text-locale-name-tables, text-locale-am-neutral, displayed-text-locale-names); 8 remain failing and are filed as T-1859 (eras, calendar digits) and T-1864 (currency with locale). The five cells of the task pass.
