---
id: T-1062
title: "Multi-area reference arguments: Areas type, reference-context evaluator, aggregates (SPEC-0004 step 2)"
milestone: M1
lane: engine
status: todo
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1061]
---

## Goal
`=SUM((A1,B2))` and the other aggregates sum, count or average the areas of a union instead of answering `#VALUE!`, through one reference-context evaluator that the later steps reuse.

## Context
SPEC-0004 step 2: add `Areas` and `RefArg` (reference_operators.rs) and `Model::evaluate_reference_arg` (util.rs), built on the existing `reference_areas`/`intersect_areas` (no new union walker). Add `for_each_cell_in_areas`, the one walker for `Areas`: each sheet's used area is looked up once per walk, and more than `MAX_AREA_CELLS` (2^24) clipped cells gives `#VALUE!` before any walk (DEVIATION, extend references.md "Union area cap"). Migrate the callers of `evaluate_aggregate_arg` that walk ranges (SUM, COUNT, COUNTA, AVERAGE, MIN, MAX, PRODUCT, STDEV/VAR families, …) to walk `Areas` area by area through it, and delete `evaluate_aggregate_arg` at the end; replace AGGREGATE's `Node::OpUnionKind` special case and `union_is_reference` (ISREF) with the evaluator. AGGREGATE's array form keeps its `Array`, `Range`, number and error arms unchanged. A 3-D reference stays `RefArg::Value` and is never put into `Areas` (`Areas::new` refuses it). `evaluate_node_in_context` must keep answering `#VALUE!` for a union (VERIFIED value-context rule). Probe rows 1, 2, 11, 15 of the spec, recorded by T-1061, are the evidence; follow them where they differ from the spec's plan.

## Acceptance criteria
- [ ] Probe rows 1, 2, 11 and 15 (T-1061's cases) pass; gen-m1/c940d3cfb0d3b083 and probes/union-* stay green.
- [ ] Unit tests: an error inside a union argument, `MAX_AREAS` areas and one more, a whole-column union `SUM((A:A,C:C))`, and `MAX_AREAS` whole-column areas over a populated sheet giving `#VALUE!` from the cell bound without walking.
- [ ] The single-area path allocates nothing new (`RefArg::Value` passes through; `single_sheet_areas()` borrows); 3-D cells of probes/3d-ref-functions stay green.
- [ ] If probe row 1 confirms it, a gen-m1 metamorphic family `SUM((r1,r2))` = `SUM(r1,r2)` for same-sheet references.
- [ ] `Areas` never reaches `evaluate_node_in_context` callers (it is not a `CalcResult` variant).
- [ ] `make check` green; reviewers approve.

## Out of scope
- AREAS, INDEX area_num (T-1063); intersection operands, LET/IF/CHOOSE (T-1064); names (T-1065); functions T-1061 files.
