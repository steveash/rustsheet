---
id: T-1921
title: "A legacy N over INDEX of a one-column array with one index is #VALUE!"
milestone: M2
lane: functions-lookup
status: in-progress
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
- [ ] probes/legacy-n-t-marker-shapes S!F10 leaves eval/baseline/probes.txt, with no regressions.
- [ ] The INDEX shape finding is recorded in docs/excel-semantics (references.md or lookup.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
