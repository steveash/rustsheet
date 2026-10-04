---
id: T-0930
title: "API edit follow-ups from the T-0103 review"
milestone: M1
lane: api
status: done
priority: P3
size: S
depends_on: [T-0103]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Close the MINOR findings deferred from the T-0103 review.

## Acceptance criteria
- [x] A test pins `Model::evaluate` being called once per `edit` (needs an engine evaluation counter or equivalent seam).
- [x] `Txn::set` skips re-snapshotting a spill footprint already logged in the same transaction; `input_seen` uses the cheap key hasher; a test covers `MAX_FOOTPRINT_CELLS` rejection.
- [x] `Before::rank` uses a sheet-id to position map instead of a linear scan.
- [x] Engine-level unit tests for `Model::take_sheet`/`restore_sheet`; sheet-name/last-sheet error classification lives next to `RefError::from_engine_message` with a test per branch (rename to a duplicate/invalid name).
- [x] Split the diff machinery out of `edit.rs` into `edit/diff.rs` (M3 replaces it with dirty-set tracking) and share the sort-by-(sheet, row, col) tail between `value_changes` and `input_changes`.
- [x] `roll_back` asserts (`debug_assert!`) that undo steps succeed; note `prepare_cell_for_user_input` and `take_sheet`/`restore_sheet` as RustSheet-only engine additions in UPSTREAM.md.
- [x] `edit_bench` moved to `crates/rustsheet/benches/edit.rs` (`cargo bench -p rustsheet --bench edit`).
- [ ] Record eval run time before/after (no more than 5% regression): deferred to T-0932.

## Out of scope
Dirty-set `ChangeSet` (M3).

## Outcome
All items done except the eval run-time comparison, deferred to T-0932 (the only engine hot-path change is one integer increment per `evaluate()`). Added `Model::evaluation_count` (RustSheet-only, listed in UPSTREAM.md) so `one_evaluation_per_edit` pins one recalculation per `edit`. Bench (noisy, concurrent runs): recalc 40-51 ms, tracked set 42-45 ms, diff overhead 9-14 ms, rollback 3 ms. `evaluation.rs` is an oversized inherited file, so comment lines were condensed to keep it from growing.
