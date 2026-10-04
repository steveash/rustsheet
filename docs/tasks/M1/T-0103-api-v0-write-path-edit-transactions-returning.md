---
id: T-0103
title: "API v0 write path: edit() transactions returning ChangeSet"
milestone: M1
lane: api
status: done
priority: P1
size: M
depends_on: [T-0102]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Implement `Workbook::edit`/`set` with rollback on error and a ChangeSet of input and value changes (computed by diffing before/after a full recalc in v0).

## Context
Per SPEC-0001; incremental ChangeSet comes in M3.

## Acceptance criteria
- [x] Atomic via undo log (no whole-model clone): a failing op inside edit leaves the workbook unchanged (test).
- [x] Bench recorded: one-cell edit and rollback on >=100k formulas; `set_track_changes(false)` opt-out implemented.
- [x] ChangeSet lists exactly the cells whose values changed (property test vs full diff).
- [x] Eval spec adapter uses `edit`.
- [x] `make check` green; reviewers approve.

- [x] Remove `Workbook::from_engine_model` and the `engine-model` feature (T-0102): the eval adapter's `evaluate_spec` builds spec workbooks through `edit`.

## Out of scope
Undo/redo, structural edits.

## Outcome
`Workbook::edit`/`set`/`set_track_changes`, `Txn`, `Input`, `ChangeSet` in `crates/rustsheet/src/edit.rs`. Atomic via an undo log (cells + spill footprint + link; sheet add/rename/delete via `Model::take_sheet`/`restore_sheet`), no workbook clone. The eval adapter builds spec workbooks through `edit`; `from_engine_model` and the `engine-model` feature are gone. Eval: 0 regressions, 0 fixed (ironcalc-calc 1632 -> 1632).

Bench (release, 100k chained formulas, `cargo test --release -p rustsheet -- --ignored --nocapture edit_bench`): recalc ~37-44 ms, one-cell `set` untracked ~30 ms vs tracked ~35 ms (diff overhead ~5 ms, 13-17% of recalc, under the 25% gate, so tracking stays on by default); failed-edit rollback ~8 ms. The first diff design (hash map of all formula cells) cost 35-40% and was replaced by a storage-order merge.

Deviations: randomized test with a deterministic generator instead of proptest (not in the offline dependency set); no direct "one evaluation per edit" counter (ops never call evaluate; sheet-level engine ops evaluate on their own, documented in SPEC-0001). Eval run time before/after moving the adapter onto `edit` was not measured separately (spec cases only; suite run time unchanged in `make check`).
