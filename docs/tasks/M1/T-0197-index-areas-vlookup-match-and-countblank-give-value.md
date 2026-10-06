---
id: T-0197
title: "INDEX, AREAS, VLOOKUP, MATCH and COUNTBLANK give #VALUE! for a 3-D reference"
milestone: M1
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-0122]
---

## Goal
INDEX, AREAS, VLOOKUP, MATCH and COUNTBLANK return `#VALUE!` for a 3-D reference argument, as Excel does.

## Context
T-0122: a 3-D reference evaluates to `#REF!` where a value is needed (`get_sheet_span_value`) and
to `#VALUE!` through `evaluate_node_with_reference` (`get_sheet_span_reference`), both in
crates/rustsheet-engine/src/reference_operators.rs. ROWS, COLUMNS, ROW, SUMIF and COUNTIF already
give `#VALUE!`; these five read their reference argument another way and give `#REF!` (COUNTBLANK
gives 0). Failing cells: probes/3d-ref-functions Calc!C1 INDEX, C5 AREAS, C6 VLOOKUP, C10 COUNTBLANK,
C13 MATCH, all expected `#VALUE!`.
Rule: docs/excel-semantics/references.md "3-D references outside the aggregates".

## Acceptance criteria
- [ ] The five cells above pass; no new F lines in eval/baseline.
- [ ] A unit test per function in crates/rustsheet-engine/src/test/test_sheet_span_references.rs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- 3-D references in the aggregates (T-0196).

## Outcome
INDEX, AREAS, VLOOKUP, MATCH and COUNTBLANK now give `#VALUE!` for a multi-sheet 3-D reference via `Model::evaluate_single_sheet_arg`; unit test added. Fixes probes/3d-ref-functions C1, C5, C6, C10, C13.
