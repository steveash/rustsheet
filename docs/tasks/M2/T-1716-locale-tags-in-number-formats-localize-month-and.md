---
id: T-1716
title: "Locale tags in number formats localize month and day names"
milestone: M2
lane: functions-text
status: todo
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
- [ ] The five failing cells above pass; probe further tags (`[$-407]` days, `[$-40C]` AM/PM, `[$-F400]`) and record them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
