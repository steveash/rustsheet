---
id: T-3706
title: "Replace the wall-clock bounds in evaluation and range-walk tests with work counts"
milestone: M5
lane: quality
status: todo
priority: P3
size: M
tier: sonnet
depends_on: [T-2282]
---

## Goal
No evaluation or range-walk test asserts an absolute wall-clock bound: once T-2282 lands, the 13
assertions left in the five files below count work, so `make check` cannot fail on a busy machine.

## Context
- Found in T-2282's quality review (#2723). T-2282 replaces the large-spill test's 20 s bound with a count of rectangle
  tests (`SeenRanges::rectangles_tested`, `#[cfg(test)]`); that counter lands with T-2282. T-2282 saw wall-clock bounds
  fail under a load average of about 15 (four `make check`s on four cores); a count is stable where time is not.
- The inventory (`rg -n "elapsed\(\)(\.as_[a-z_]+\(\))?\s*<|Duration::from" crates/rustsheet-engine/src`, 2026-11-01) is 30 bound
  assertions in 16 test files on main, 29 once T-2282 removes the large-spill one, split by what they guard into T-3706 (evaluation and range walks), T-3707 (regex and
  PCRE backtracking) and T-3708 (function kernels). `test/test_recalc_budget.rs` `worked_example_timing` only prints its
  timing and asserts no bound, so none of the three covers it; `clock.rs` uses durations as inputs, not bounds.
- Reuse a natural counter (the area budget, the recalc budget, a step budget, a loop bound) before adding one; any new
  counter is `#[cfg(test)]` or `pub(crate)`, never public.
- This task's tests:
  - `test/test_clipped_range_walks.rs` (8 assertions);
  - `test/dynamic_evaluation/test_range_reads.rs` (2, after T-2282 replaces the large-spill test's);
  - `test/test_sheet_span_references.rs` `whole_columns_walk_only_the_used_area` (10 s);
  - `test/test_defined_name_formulas.rs` `branching_names_are_bounded_in_time` (10 s);
  - `expressions/parser/tests/test_add_implicit_intersection.rs` `nested_if_conditions_stay_linear` (5 s).
- The area budget (`area_budget.rs`) already counts the cells and rectangles a formula walks; prefer it for the walk
  tests.

## Acceptance criteria
- [ ] `rg -n "elapsed\(\)(\.as_[a-z_]+\(\))?\s*<|Duration::from" crates/rustsheet-engine/src/test/test_clipped_range_walks.rs crates/rustsheet-engine/src/test/dynamic_evaluation/test_range_reads.rs crates/rustsheet-engine/src/test/test_sheet_span_references.rs crates/rustsheet-engine/src/test/test_defined_name_formulas.rs crates/rustsheet-engine/src/expressions/parser/tests/test_add_implicit_intersection.rs` finds no bound assertion; each guard asserts a work count
      or a ratio of work counts instead (never a ratio of timings).
- [ ] Each still fails on a deliberate local regression of the code it guards (described in the Outcome).
- [ ] No test is skipped, ignored or deleted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the code under test. Benchmarks (`benches/`) keep their timings.
- The other two tasks' files.
