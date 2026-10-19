---
id: T-1943
title: "Probe the shape of INDEX's whole-row and whole-column picks of a scalar or a 1x1 array"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/lookup.md "INDEX of an array: a whole row stays an array" says whether a whole-row or whole-column
pick (an index of 0) of a scalar or a 1x1 array is a scalar or a 1x1 array, from Excel evidence.

## Context
T-1921 (probes/index-one-index-array-shape) showed that `INDEX({2;3},1)` and `INDEX({2;3},1,0)` are 1x1 arrays (`TYPE` 64),
and that `INDEX({2},1)` and `INDEX({2,3},1)` are scalars. The engine now returns an array for every pick by a 0 index
(`index_from_array`), including `INDEX({7},0,0)`, `INDEX({7},1,0)`, `INDEX({7},0)` and `INDEX(5,0,0)`. Their values are probed
(probes/index-scalar-indexes) but their shape is not. Probe with `TYPE(...)` in a dynamic cell, and with a legacy
`=N(INDEX({7},0,0))` (the SPEC-0012 marker reads an array as `#VALUE!`). Cover `INDEX({7},0,0)`, `(…,1,0)`, `(…,0,1)`,
`INDEX({7},0)`, `INDEX(5,0,0)`, `INDEX(5,0)` and `INDEX({2,3},0)`. Then record the result in lookup.md. If the engine
differs, file a functions-lookup fix task.

## Acceptance criteria
- [ ] A probe case with excel-graph values, tagged M2, covers the shapes above.
- [ ] lookup.md records the result. A fix task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
