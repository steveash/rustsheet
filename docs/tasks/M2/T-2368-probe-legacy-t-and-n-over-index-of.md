---
id: T-2368
title: "Probe legacy T and N over INDEX of a text, boolean or negative scalar with a 0 index"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's legacy `=T(INDEX(5,0,0))`, `=N(INDEX("a",0,0))`, `=N(INDEX(TRUE,0))` and `=N(INDEX(-5,0,0))` are probed, so the engine's `indexes_a_number_whole` (T-2191) stops guessing.

## Context
T-2191 made legacy N over `INDEX(<number literal>,0[,0])` `#VALUE!` with no `@` (probes/index-zero-index-scalar-shape, N only). T shares the code path and the text, boolean, negative (unary minus) and empty-argument (`INDEX(5,,0)`) forms keep the old `@` behavior; none is probed. Code: `indexes_a_number_whole` in expressions/parser/static_analysis/legacy.rs.

## Acceptance criteria
- [ ] A probes case holds the forms above with `=FORMULATEXT` beside each, filled by excel-graph (excel-probe skill).
- [ ] docs/excel-semantics/lookup.md "INDEX of an array: a whole row stays an array" records the result; the engine matches it or a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
