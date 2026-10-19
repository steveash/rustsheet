---
id: T-1946
title: "Probe MAP with a blank or error later array and INDEX of a LAMBDA out of range"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/arrays.md "MAP over single values" and "An uncalled LAMBDA is one value" cover, from Excel evidence,
the shapes T-1511 left unprobed.

## Context
T-1511 (probes/lambda-value-arguments) found that MAP refuses a single number, text or boolean as its second or later
array (`#VALUE!`). The engine also refuses a blank there, for example `MAP({1},,LAMBDA(x,y,x))` or `MAP({1},Z9,..)` with an
empty Z9 (a reference, so likely allowed), but lets an error value through (`MAP({1},NA(),..)`). Neither is probed. INDEX of
a LAMBDA with an index of 2 or a negative index is `#REF!` in ours, also unprobed. The cell `=MAP(1,LAMBDA(x,y,x))`
(lambda-value-arguments B6) got no excel-graph value; read it through `ERROR.TYPE(..)`. Probe these cells and record the
results. If the engine differs, file a functions-logic fix task. Code: `by_slices_source` / `is_scalar_value`
(bycol_byrow.rs), and the LAMBDA arm of `index_select` (index.rs).

## Acceptance criteria
- [ ] A probe case tagged M7 with excel-graph values covers the shapes above.
- [ ] arrays.md records the results. A fix task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
