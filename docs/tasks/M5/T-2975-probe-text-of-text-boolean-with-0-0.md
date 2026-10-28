---
id: T-2975
title: "Probe TEXT of text/boolean with ;;;, 0;0;0; and five-section formats"
milestone: M5
lane: functions-text
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's result for `TEXT` of a text or boolean value is probed for the formats T-2945 left unevidenced, and the formatter follows it.

## Context
T-2945 fixed `TEXT("text",";;;")` (evidence: gen-m10/0e3d8eba9432a18d T!A24) without oracle access for the rest. Unevidenced: `=TEXT(TRUE,";;;")`, `=TEXT("x","0;;;;")` (five sections: `#VALUE!` for numbers, unknown for text), `=TEXT("x","0;0;0;")` (non-empty number sections, empty 4th; `Parser::has_empty_text_section` returns empty text) and `=TEXT("x","0;0;General;")`. Code: `format_text_value` in formatter/format.rs; the note in docs/excel-semantics/number-formats.md "Text and booleans use the format's text section".

## Acceptance criteria
- [ ] A probe case holds the four formulas above on text and boolean inputs with Excel values (`excel-probe` skill), and the formatter matches them.
- [ ] number-formats.md no longer calls these cases unprobed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
