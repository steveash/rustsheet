---
id: T-2084
title: "Spec cases: bake rendered pivot cells and load pivot_tables into the engine adapter (SPEC-0015)"
milestone: M6
lane: harness
status: todo
priority: P3
size: M
tier: sonnet
depends_on: [T-2081]
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
Spec cases with `pivot_tables` reach the engine adapter with their rendered cells and pivot, so probes/getpivotdata-real-pivot scores against the engine.

## Context
SPEC-0015 "Spec cases in the eval adapter": add `rendered` to `PivotTableSpec` (crates/rustsheet-eval/src/case.rs), a `rustsheet-spec bake-pivots` command in tools/ that fills it from tools/src/rustsheet_tools/pivot.py's layout, a spec2xlsx check that `rendered` equals the layout, and a facade `Txn::add_pivot_table(PivotTableInput)` the adapter calls after values (builds the cache from the source values). Depends on T-2081; with T-2083 the case passes.

## Acceptance criteria
- [ ] probes/getpivotdata-real-pivot is baked and, with T-2083 merged, passes all cells (baseline shrinks by 22).
- [ ] `Txn::add_pivot_table` is `#[doc(hidden)]` test support with a typed `EditError::Pivot`; the Python binding defers it.
- [ ] A probe with oracle values for the rendered label, caption, blank and grand-total cells the GETPIVOTDATA probes read (A5, C3, ...), so the baked layout rests on committed evidence.
- [ ] spec2xlsx rejects a case whose `rendered` disagrees with the layout (test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).
