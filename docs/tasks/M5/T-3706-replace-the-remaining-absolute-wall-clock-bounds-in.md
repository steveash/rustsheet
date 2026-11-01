---
id: T-3706
title: "Replace the remaining absolute wall-clock bounds in engine tests with work counts"
milestone: M5
lane: quality
status: todo
priority: P3
size: M
tier: sonnet
depends_on: [T-2282]
---

## Goal
No engine test asserts an absolute wall-clock bound: each guard against a quadratic or unbounded regression counts
work (steps, cells, nodes visited) or compares a small and a large input, so `make check` cannot fail on a busy machine.

## Context
- Found in T-2282's quality review (#2723): T-2282 replaced the large-spill test's 20 s bound with a count of rectangle
  tests (`SeenRanges::rectangles_tested`, `#[cfg(test)]`). Six other tests still assert elapsed time:
  - `expressions/parser/tests/test_add_implicit_intersection.rs` `nested_if_conditions_stay_linear` (5 s);
  - `test/test_sheet_span_references.rs` `whole_columns_walk_only_the_used_area` (10 s);
  - `test/test_fn_depreciation.rs` `fn_vdb_huge_period_is_capped` (5 s);
  - `test/test_defined_name_formulas.rs` `branching_names_are_bounded_in_time` (10 s);
  - `test/test_recalc_budget.rs` `worked_example_timing`;
  - `test/mathematical/test_multinomial.rs` `test_multinomial_overflow_num_error` (5 s).
- T-2282 saw such tests fail under a load average of about 15 (four `make check`s on four cores). A count is stable
  where time is not. Where a natural counter exists (the area budget, the recalc budget, a loop bound), reuse it before
  adding a `#[cfg(test)]` one.

## Acceptance criteria
- [ ] None of the six tests asserts an absolute wall-clock bound; each asserts a work count or a size ratio instead.
- [ ] Each still fails on a deliberate local regression of the code it guards (described in the Outcome).
- [ ] No test is skipped, ignored or deleted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the code under test. Benchmarks (`benches/`) keep their timings.
