---
id: T-1194
title: "Name lookups in reference context: one lowercase lookup per use, and look through formula-bodied names in is_multi_area"
milestone: M1
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A defined name used in reference context is looked up once per use, and a name whose formula body cannot stand for several areas (`=0.5`, `=OFFSET(...)`) keeps the allocation-free intersection path, with the same results.

## Context
Performance review of T-1065 (MINOR, deferred). `Model::is_multi_area` (reference_operators.rs) is
true for every name with a `Formula` body, through `Model::defined_name_formula` (defined_names.rs),
which lowercases the name and does a HashMap lookup each time. A name in reference context is now
looked up up to three times (`is_multi_area` in the caller, `defined_name_reference`, then
`evaluate_node_with_reference` for a body that is not multi-area), each with its own lowercase
allocation; COUNT/COUNTA arguments, `bind_argument`, ROWS/COLUMNS, FREQUENCY and every intersection
call `is_multi_area`. Options: store the lowercased key in the `DefinedNameKind` node at parse time,
or resolve once and pass the `Arc<Node>` down. Also look through the body one level in
`is_multi_area` (names inside the body counted as multi-area without looking, so recursion stays
bounded), so `dyn A1` with `dyn` = `=OFFSET(...)` does not build an area list.

## Acceptance criteria
- [ ] No change in eval results; the test_multi_area_arguments.rs pair-path test covers a formula-bodied scalar name.
- [ ] At most one name-key allocation per name use in reference context (shown by the code path, or a counting test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The value path's lookup in model.rs (same pattern; may share the fix).
