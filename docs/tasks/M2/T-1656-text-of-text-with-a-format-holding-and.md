---
id: T-1656
title: "TEXT of text with a format holding @ and a date token (x@y) is #VALUE!"
milestone: M2
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT of a text value with a one-section format holding `@` and a date token is `#VALUE!`, as in Excel.

## Context
probes/text-lone-surrogates A91 (`LEN(TEXT("😀","x@y"))`) is `#VALUE!` in Excel (excel-graph); the engine returns 3. A85, A89, A90, A92, A94 fail the same way. Valid quoted formats (`"x"@"y"`, A97-A101) already pass. Probe first which tokens beside `@` error (`y`, `d`, `m`, `h`, `s`, `b`, `e`, `g`) for text values; see docs/excel-semantics/text.md (lone surrogates) and number-formats.md.

## Acceptance criteria
- [x] Probe cells for the date-token set beside `@` on text, with oracle values; the engine matches them and A85, A89-A92, A94 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-


## Outcome
TEXT now rejects a format, for any value, when `@` stands in a section other than the last, or beside a number or date token (`exceeds_complexity_limit`, formatter/complexity.rs). A last `@` section is the text section in 2- and 3-section formats too, and numbers use the sections before it (format.rs). In the text section, `*x` prints nothing and `AM/PM` prints "AM". New probe probes/text-at-token-sections (96 cells, excel-graph) settles how sections interact; the handoff's 4-section question turned out to be the same placement rule. Eval: probes 6 fixed (text-lone-surrogates A85, A89-A92, A94), 2 new cases (181 of 185 cells pass), 0 regressions; poi 63934.xlsx B2 drops out (stale, already fixed on main). The 4 remaining cells (`A/P`, `[$-409]`) are lexer gaps, filed as T-1702.
