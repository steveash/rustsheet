---
id: T-0229
title: "Probe AND/OR/XOR with several errors and errors inside the deciding range (which error wins)"
milestone: M2
lane: functions-logic
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Probe which error AND/OR/XOR return when several arguments are errors, and when the error sits in the same range as the deciding value; record the result.

## Context
T-0215 made AND/OR evaluate every argument; the engine returns the first error left to right, row by row. Unprobed: `OR(TRUE,NA(),1/0)`, a 2x2 range with errors in different rows/columns, `OR(B1:B2)` and `OR(TRUE,B1:B2)` with an error in the range.

## Acceptance criteria
- [ ] Probe evidence cited in docs/excel-semantics/coercion.md; engine matches (fix or confirm); unit tests; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
