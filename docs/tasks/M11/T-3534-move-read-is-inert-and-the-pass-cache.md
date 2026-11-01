---
id: T-3534
title: "Move read_is_inert and the pass-cache read rule out of ifs_cache.rs to a neutral home shared by the criteria and SUM prefix caches"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The rule that lets a pass cache skip a cell read (`read_is_inert`, T-0449) lives in one neutral place
beside the pass-cache plumbing of `Evaluation`. So does the per-pass range-values cache it guards
(`ifs_range_values`, `RangeView`, `RectKey`). Every pass cache uses them from there: criteria, SUMPRODUCT
blocks, operator range operands and SUM prefixes.

## Context
T-3456 (`functions/math_and_trigonometry/sum_prefix.rs`) widened `read_is_inert` in
`functions/statistical/ifs_cache.rs` to `pub(crate)`, so a math function now depends on a statistical cache
file (architecture review of T-3456, MINOR). T-3471 then made core evaluation (`evaluate_range` in
`cast.rs`) read `ifs_range_values`. Its architecture review (MINOR) asked to widen this task to move the
range-values cache too. `pass_cache_safe` already lives in `evaluation.rs`.

## Acceptance criteria
- [x] `read_is_inert` and the range-values cache are defined once outside `functions/statistical/`.
      `ifs_cache.rs`, `sumproduct.rs`, `cast.rs` and `sum_prefix.rs` call them there. Behavior is
      unchanged: the existing cache tests pass, and only the test of T-0498's counts changes its call
      shape.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing what counts as inert.

## Outcome
- **New module:** `range_values.rs` (crate root, beside `evaluation.rs`) now holds the shared pass-cache
  plumbing:
  - `MIN_CACHED_CELLS`, `MAX_CACHED_CELLS` and `MAX_PARENTS`;
  - `RectKey`, `rect_of`, `rect_cell` and `rect_size`;
  - `RangeState` and `RangeView`;
  - `Model::read_is_inert`;
  - `Model::range_values`, renamed from `ifs_range_values`;
  - a `RangeValues` store: the cached ranges, their parents for block reads, and the pass's cells budget.
- **Budget:** the budget is shared through `RangeValues::charge` and `fits`. `ifs_cache.rs`'s indexes, block
  numbers and gathered values charge it there, as they charged `IfsCache::cells` before.
- **Wiring:** `Evaluation::range_values` is cleared in `clear_pass_caches` with the other pass caches.
- **What stays in `ifs_cache.rs`:** only the criteria-specific state: match sets, indexes, block numbers,
  chains and gathered values. It went from 981 to 803 lines.
- **Tests:** `IfsCache::counts` takes the `RangeValues` for its ready and waiting counts, and the one test that
  calls it passes it. No other test changed.
- **Review fixes:** the four reviewers caught slips in doc comments.
  - A misplaced `Evaluation` field doc: each field now has its own.
  - Two stale names: `ifs_range_values` in `ifs_cache.rs`, and the test's `ifs_cache::MIN_CACHED_CELLS`.
  - `MIN_CACHED_CELLS` and `rect_size` are now private to the module.
- **Left as is:** `sumproduct.rs` still takes `BlockNumbers` from `ifs_cache.rs`. Those numbers are derived
  from a criteria-cache entry and cached with it, so they stay with that cache.
