---
id: T-3532
title: "Implement numeral-system digits of locale tags (probe t1925)"
milestone: M10
lane: functions-text
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Dates and plain numbers under a locale tag with a numeral byte (bits 24-31) print the native digits Excel prints.

## Context
Split from T-1925, which probed it. Table and quirks (no zero in Tamil and Ethiopic, CJK numerals 27-31 with 十 in dates): docs/excel-semantics/number-formats.md "Numeral-system digits"; cases: probes/t1925-numeral-digits (all cells). Code: formatter/locale_tag.rs `resolve_lcid` returns Silent for any numeral byte; the tag must reach the date and number parts (parser.rs `DatePart`/number part) and a post-pass or digit emitter must map digits. A numeral byte must not drop the LCID's names or era (B column). Unprobed: digits in quoted literals and in text sections.

Also owned (gap triage 2026-11-01; same numeral byte, unowned by cell until now):
- probes/t1924-locale-tag-extended-hex.case.json: S!A26, S!B26 (`[$-fr_FR-x]`, Myanmar digits "၂၀၂၃", "၆ P")
- probes/text-locale-calendars.case.json: S!A26 (`TEXT(45000,"[$-8000411]g|gg|ggg|e|ee|yyyy|mmmm")` is
  "R|令|令和|୫|୦୫|୦୫|3月": Odia digits and the Japanese era kept; ours "|||2023|2023|2023|March")

## Acceptance criteria
- [ ] probes/t1925-numeral-digits cells in columns A, B and D pass (column C also needs T-3598), baseline shrinks; `[$-fr_FR-x]` Myanmar digits (t1924 A26, B26) pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
