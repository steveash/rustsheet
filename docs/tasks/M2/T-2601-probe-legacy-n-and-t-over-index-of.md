---
id: T-2601
title: "Probe legacy N and T over INDEX of a scalar with an empty column or a unary plus or parenthesised scalar"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's legacy `=N(INDEX(5,0,))`, `=N(INDEX(5,))`, `=N(INDEX(+5,0,0))` and `=N(INDEX((5),0,0))` (and T, and a negated text or boolean) are probed, so `indexes_a_scalar_whole` stops guessing about them.

## Context
T-2368 probed the scalar literals with a 0 index (probes/index-zero-index-scalar-t-n); the forms above keep the old `@` behavior and are unprobed. Code: `indexes_a_scalar_whole` in expressions/parser/static_analysis/legacy.rs.

## Acceptance criteria
- [ ] A probes case holds the forms above with `=FORMULATEXT` beside each, filled by excel-graph with the cells in the sheet's `legacy_formulas` (excel-probe skill).
- [ ] docs/excel-semantics/lookup.md "INDEX of an array: a whole row stays an array" records the result; the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
