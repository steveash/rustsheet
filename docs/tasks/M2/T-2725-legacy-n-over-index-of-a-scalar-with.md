---
id: T-2725
title: "Legacy N over INDEX of a scalar with a nonzero index shows no @: FORMULATEXT of =N(INDEX(5,1)) is =N(INDEX(5,1))"
milestone: M2
lane: functions-lookup
status: done
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
whole (`indexes_a_scalar_literal` in expressions/parser/static_analysis/legacy.rs, `#VALUE!`), but a nonzero index must keep
the element's value with no marker. Unprobed: T, text and boolean scalars, `INDEX(5,1,1)`, `INDEX(5,2)` (`#REF!`?) and a
computed index. Probe those first (excel-probe skill).

## Acceptance criteria
- [x] probes/t2601-index-scalar-forms-t-n B28 passes and leaves eval/baseline/probes.txt; new probes pass.
- [x] lookup.md "INDEX of an array: a whole row stays an array" records it.
- [x] `make check` green; reviewers approve.

## Out of scope
- The unary plus in formula text (T-2723).

## Outcome
Done (probes/t2725-index-scalar-nonzero-t-n A1:B24, excel-graph 2026-10-25). Excel shows no `@` on INDEX of a scalar
literal under a legacy N or T, whatever the indices. Nonzero indices take the element (5, "a", `#REF!` past it, `#VALUE!`
below 1), and any 0 or empty index takes the whole scalar (`#VALUE!`). `indexes_a_scalar_literal` now depends only on the
first argument being a scalar literal, under groups and minus; INDEX itself gives the values. An index cell holding 0 or blank is `#VALUE!` like a literal 0. Array, range and IF indices keep their
own `@` and take the element (A17:A24). probes/t2601 B28 is fixed.
