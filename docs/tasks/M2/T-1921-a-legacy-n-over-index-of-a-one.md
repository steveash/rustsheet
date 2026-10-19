---
id: T-1921
title: "A legacy N over INDEX of a one-column array with one index is #VALUE!"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A legacy (non-array) cell `=N(INDEX({2;3},1))` gives `#VALUE!`, as Excel does, instead of 2.

## Context
probes/legacy-n-t-marker-shapes S!F10 (excel-graph, 2026-10-18, T-1713 review). The legacy N/T marker
(SPEC-0012, T-1713) marks this call, since INDEX's `array` is an array constant, but the marked kernel only
gives `#VALUE!` for an `Array` argument, and our INDEX of a one-column array with one index returns the
scalar 2. Excel's values from the same probe:
- `=N(INDEX({2;3},1))`: `#VALUE!` (F10), FORMULATEXT with no `@`.
- `=N(INDEX({2,3;4,5},1))`: `#VALUE!` (E10). Ours matches: a row of a 2x2 is an array.
- `=N(INDEX({2},1,1))`: 2 (G10). Ours matches.

So Excel seems to read `INDEX({2;3},1)` as a 1x1 array (the first row), and the explicit row and column
form as a scalar. That is unprobed for a dynamic cell. Probe first: dynamic `=INDEX({2;3},1)` with
`=ROWS(...)`, `=TYPE(...)` and `=ISREF(...)`, and `INDEX({2,3},1)` (a one-row array). Then decide whether
INDEX returns a 1x1 array or whether only the legacy kernel needs the shape. docs/excel-semantics/references.md
"Legacy N and T over a reference-returning call".

## Acceptance criteria
- [x] probes/legacy-n-t-marker-shapes S!F10 leaves eval/baseline/probes.txt, with no regressions.
- [x] The INDEX shape finding is recorded in docs/excel-semantics (references.md or lookup.md).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/index-one-index-array-shape (excel-graph, 2026-10-19) shows that INDEX itself returns the 1x1 array. Dynamic
`TYPE(INDEX({2;3},1))` is 64, as is `TYPE(INDEX({2;3},2))`. `TYPE(INDEX({2,3},1))`, `TYPE(INDEX({2},1))` and
`TYPE(INDEX({2;3},1,1))` are 1. A legacy `=N(INDEX({2;3},1,0))` is `#VALUE!`. `index_from_array` now returns a pick by a whole
row or column as an array, even when that pick is one cell; only a pick by a row number and a column number is a scalar.
A one-row array's single index is a column number into row 1. References are unchanged.
lookup.md has a new entry, "INDEX of an array: a whole row stays an array", and references.md notes F10.
Eval delta: probes known failures -1 (legacy-n-t-marker-shapes S!F10). The new case index-one-index-array-shape passes
36/36. There are no regressions. Follow-up: T-1943 probes the shape of a 0-index pick of a scalar or a 1x1 array.
