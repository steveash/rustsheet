---
id: T-1377
title: "HLOOKUP/VLOOKUP pick is a reference when the index argument is IF(..) picking a cell (follow-up of T-0299)"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`COUNT(HLOOKUP(42,F5:G6,IF(TRUE,G5)))` is 0 like the plain-cell index: an `IF` index that picks a cell leaves the lookup pick a reference.

## Context
Probe `probes/lookup-pick-index-shapes` (T-0299), excel-graph: Sheet1!A3, A9 and B2 are 0, the engine gives 1 (the index `IF` evaluates to a value, so the pick is a plain value). INDEX, CHOOSE, OFFSET and a name already match. See `lookup_index_arg` (T-0295) and docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] `probes/lookup-pick-index-shapes` Sheet1!A3, A9, B2 pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
