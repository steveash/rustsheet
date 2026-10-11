---
id: T-1264
title: "XLOOKUP with an array lookup_value looks up each element"
milestone: M1
lane: functions-logic
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XLOOKUP with an array `lookup_value` (`SUM(XLOOKUP({1;2},A1:A3,B1:B3))`) returns one result per element, as Excel does.

## Context
probes/let-name-uncalled-picked-refs Sheet1!H21: Excel gives 30, the engine `#N/A` (`xlookup_impl`, functions/xlookup.rs, evaluates lookup_value to a single value). Probe the shapes first (row/column/2-D lookup_value, with if_not_found and the reference form) with the excel-probe skill, and record them in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [ ] H21 passes and its baseline entry is removed; probed shapes are recorded in docs/excel-semantics/lookup.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
XLOOKUP lifts an array or multi-cell `lookup_value` per element through `lift_with_cost` (functions/xlookup.rs); a wide return row keeps its first cell per element. New probe probes/xlookup-array-lookup-value (27 cells, all pass); let-name-uncalled-picked-refs H21 now passes. Shapes recorded in docs/excel-semantics/lookup.md.

## Handoff
Branch has a working implementation (XLOOKUP lifts an array/multi-cell `lookup_value` through `lift_with_cost`, wrapping the scalar path as `xlookup_scalar`), probe `probes/xlookup-array-lookup-value` (27 oracle cells) and the lookup.md entry; `make check` was green with the baseline updated. Reviews: architecture APPROVE; security, quality, performance REQUEST_CHANGES.
- Security (MAJOR): `xlookup_walk_cost` charges 1 for a computed-array lookup_array and ignores return_array/if_not_found, so `=XLOOKUP(ROW(1:1000000),SEQUENCE(100000),SEQUENCE(100000))` is about 1e11 operations. Needs a DEVIATION entry (decision 0014) for the refusal and a test.
- Quality (MAJOR): do not fork `Model::walk_cost` (if_ifs.rs); extend it (e.g. count computed arrays) instead.
- Performance (MAJOR): each distinct element re-evaluates args[1..5], re-clips and rebuilds the search vector, so the work is O(elements x used cells). Restructure like `lookup_each.rs` (T-0270): evaluate and validate arguments once, build the SearchVector once, run only `linear_search`/`binary_hit` plus `xlookup_pick` per element, and cost with `SearchVector::search_cost()`. The current range-size cost also falsely refuses `XLOOKUP(A:A,B:B,C:C)` with about 128+ distinct values; add a unit test for that case.
- Next: do the restructure, then rerun `make check` and all reviewers.
