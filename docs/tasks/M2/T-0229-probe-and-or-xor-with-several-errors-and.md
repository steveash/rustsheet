---
id: T-0229
title: "Probe AND/OR/XOR with several errors and errors inside the deciding range (which error wins)"
milestone: M2
lane: functions-logic
status: done
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
- [x] Probe evidence cited in docs/excel-semantics/coercion.md; engine matches (fix or confirm); unit tests; `make check` green.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/and-or-xor-error-order (30 cells, excel-graph): AND, OR and XOR return the first error, arguments left to right
and ranges/arrays row by row, whatever value would decide the result. The engine already matches every cell (confirmed, no
code change); unit test `first_error_wins_row_by_row` pins it. Recorded in docs/excel-semantics/coercion.md.
Eval delta: probes +1 case, 30/30 cells pass, 0 regressions.
