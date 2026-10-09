---
id: T-1212
title: "CODE and VALUETOTEXT lift over a range argument; VALUETOTEXT(TRUE,1) is TRUE"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`CODE` and `VALUETOTEXT` take a range in value mode, spilling one result per cell like LEN and EXACT, and `VALUETOTEXT(TRUE,1)` is "TRUE".

## Context
From T-1087's probe probes/text-argument-modes: `CODE(B1:B2)` (F5:F6) is 97, 120 in Excel but `#VALUE!` in the engine; `VALUETOTEXT(A1:A3)` (F17:F19) is "5", "ab", "TRUE" but the engine returns the text "#VALUE!"; `VALUETOTEXT(A3,1)` with A3 = TRUE (C31) is "TRUE", the engine `#ERROR!`. Semantics: docs/excel-semantics/text.md "Text function argument modes".

## Acceptance criteria
- [ ] probes/text-argument-modes C31, F5:F6 and F17:F19 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
