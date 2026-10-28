---
id: T-2601
title: "Probe legacy N and T over INDEX of a scalar with an empty column or a unary plus or parenthesised scalar"
milestone: M2
lane: functions-lookup
status: done
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
- [x] A probes case holds the forms above with `=FORMULATEXT` beside each, filled by excel-graph with the cells in the sheet's `legacy_formulas` (excel-probe skill).
- [x] docs/excel-semantics/lookup.md "INDEX of an array: a whole row stays an array" records the result; the engine matches it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done (probes/t2601-index-scalar-forms-t-n A1:B28, excel-graph 2026-10-25). Every probed form under a legacy N or T is
`#VALUE!` with no `@`. The forms cover an empty column or row argument (`INDEX(5,0,)`, `INDEX(5,)`, `INDEX(5,,)`), a unary
plus, and any chain of groups and minus around a number, text or boolean literal (`(5)`, `-"a"`, `--5`, `-(5)`, `((5))`,
`(TRUE)`, `(-"a")`). A grouped 0 index (`INDEX(5,(0))`) behaves the same. `indexes_a_scalar_whole` now takes all of these.
Two cells are left:
- FORMULATEXT keeps a unary plus on text and booleans and drops it on numbers; ours drops both (B14, B15, T-2723).
- `=N(INDEX(5,1))` is 5 with no `@`; ours shows an `@` (B28, T-2725).
