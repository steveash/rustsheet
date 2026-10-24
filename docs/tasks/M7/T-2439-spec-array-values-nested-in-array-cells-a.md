---
id: T-2439
title: "Spec: array values nested in array cells (a GROUPBY whose function returns an array keeps it per cell)"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec (docs/specs/) for how the engine represents and evaluates an array held inside one cell of an array result, as Excel does for a GROUPBY whose function returns an array, so T-2398's cells can be matched.

## Context
probes/scan-over-groupby (excel-graph, 2026-10-23, T-2398): `INDEX(GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,w)),4,2)` spills the
Total row's {1;2;3;4} (A69:A72), so each value cell of that GROUPBY holds its group's array. Through SCAN MIN the result is
`#N/A` (A36; probes/groupby-lifted-args A153/I153; gen-m7/b24ce74f08d3f91a A1850). `ArrayNode`
(crates/rustsheet-engine/src/expressions/parser/mod.rs) has no nested-array variant, so ours is `#VALUE!` in those cells.
docs/excel-semantics/arrays.md "SCAN over a GROUPBY result" has the observations. The spec should cover:
- what each consumer does with a nested array: INDEX, SCAN, MAP, REDUCE, the display of the cell (likely `#CALC!`) and spill;
- which other functions produce one (PIVOTBY, BYROW?), to be probed;
- memory bounds (directive 5).

## Acceptance criteria
- [ ] docs/specs/SPEC-XXXX exists with the probe evidence, the representation, the consumers and the limits.
- [ ] Implementation tasks are split from it; T-2398 depends on the one that makes its cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing it (the split tasks).
