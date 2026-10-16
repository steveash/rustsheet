---
id: T-1656
title: "TEXT of text with a format holding @ and a date token (x@y) is #VALUE!"
milestone: M2
lane: functions-text
status: todo
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
- [ ] Probe cells for the date-token set beside `@` on text, with oracle values; the engine matches them and A85, A89-A92, A94 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
