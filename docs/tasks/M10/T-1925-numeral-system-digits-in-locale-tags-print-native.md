---
id: T-1925
title: "Numeral-system digits in locale tags print native digits"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The numeral-system digits of a locale tag (bits 24-31, `[$-8000411]`) print the native digits Excel prints.

## Context
`TEXT(45000,"[$-8000411]g|gg|ggg|e|ee|yyyy|mmmm")` is "R|令|令和|୫|୦୫|୦୫|3月", Odia digits for the era year (probes/text-locale-calendars A26). The resolver returns `Silent` for any numeral digit. Probe which numbers they change: date fields, plain numbers (`[$-D000409]0.00`), and digits 1-0x1F. Code: formatter/locale_tag.rs `resolve_lcid`.

## Acceptance criteria
- [ ] A probe covers the numeral digits; its cells pass or are filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
