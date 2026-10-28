---
id: T-2975
title: "Probe TEXT of text/boolean with ;;;, 0;0;0; and five-section formats"
milestone: M5
lane: functions-text
status: done
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
- [x] A probe case holds the four formulas above on text and boolean inputs with Excel values (`excel-probe` skill), and the formatter matches them.
- [x] number-formats.md no longer calls these cases unprobed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe probes/text-format-text-sections (99 cells, excel-graph 2026-10-28): text, TRUE, FALSE, "" and numbers under
  `;;;`, `0;;;;`, `0;0;0;`, `0;0;General;`, `0;0;0`, `0;0;0;@`, `0;;;;@`, `0;0;0;;` and `;;;@;`. An empty 4th section
  gives "" for every non-number value; a format of five or more sections is `#VALUE!` for every value; `0;0;0` and
  `0;0;0;@` leave text and booleans unchanged. Ours already matched all 99 cells, so no code change: number-formats.md
  and the `empty_text_section_is_empty` comment now cite the probe.
- Eval delta: probes +1 case, 99/99, no regressions.
