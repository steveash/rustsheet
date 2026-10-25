---
id: T-2725
title: "Legacy N over INDEX of a scalar with a nonzero index shows no @: FORMULATEXT of =N(INDEX(5,1)) is =N(INDEX(5,1))"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy N or T over INDEX of a scalar with a nonzero index keeps its value and shows no `@`, as Excel does.

## Context
Found by T-2601. A legacy `=N(INDEX(5,1))` is 5 in Excel and ours, but Excel's FORMULATEXT is `=N(INDEX(5,1))` while ours is
`=N(@INDEX(5,1))` (probes/t2601-index-scalar-forms-t-n B28, excel-graph 2026-10-25). A 0 or empty index reads the scalar
whole (`indexes_a_scalar_whole` in expressions/parser/static_analysis/legacy.rs, `#VALUE!`), but a nonzero index must keep
the element's value with no marker. Unprobed: T, text and boolean scalars, `INDEX(5,1,1)`, `INDEX(5,2)` (`#REF!`?) and a
computed index. Probe those first (excel-probe skill).

## Acceptance criteria
- [ ] probes/t2601-index-scalar-forms-t-n B28 passes and leaves eval/baseline/probes.txt; new probes pass.
- [ ] lookup.md "INDEX of an array: a whole row stays an array" records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The unary plus in formula text (T-2723).
