---
id: T-0930
title: "API edit follow-ups from the T-0103 review"
milestone: M1
lane: api
status: in-progress
priority: P3
size: S
depends_on: [T-0103]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Close the MINOR findings deferred from the T-0103 review.

## Acceptance criteria
- [ ] A test pins `Model::evaluate` being called once per `edit` (needs an engine evaluation counter or equivalent seam).
- [ ] `Txn::set` skips re-snapshotting a spill footprint already logged in the same transaction; `input_seen` uses the cheap key hasher; a test covers `MAX_FOOTPRINT_CELLS` rejection.
- [ ] `Before::rank` uses a sheet-id to position map instead of a linear scan.
- [ ] Engine-level unit tests for `Model::take_sheet`/`restore_sheet`; sheet-name/last-sheet error classification lives next to `RefError::from_engine_message` with a test per branch (rename to a duplicate/invalid name).
- [ ] Split the diff machinery out of `edit.rs` into `edit/diff.rs` (M3 replaces it with dirty-set tracking) and share the sort-by-(sheet, row, col) tail between `value_changes` and `input_changes`.
- [ ] `roll_back` asserts (`debug_assert!`) that undo steps succeed; note `prepare_cell_for_user_input` and `take_sheet`/`restore_sheet` as RustSheet-only engine additions in UPSTREAM.md.
- [ ] Record eval run time before/after the adapter move (no more than 5% regression); move `edit_bench` into a bench suite.

## Out of scope
Dirty-set `ChangeSet` (M3).
