---
id: T-0932
title: "Record eval run time and a clean edit-bench baseline"
milestone: M1
lane: api
status: done
priority: P3
size: S
depends_on: [T-0930]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Close the one acceptance item T-0930 deferred, and make the edit bench usable as evidence.

## Acceptance criteria
- [x] `make eval` wall time recorded on `main` before and after the T-0930 changes (no more than 5% regression), noted in this file.
- [x] `crates/rustsheet/benches/edit.rs` runs several iterations with a warm-up and reports a median; a clean baseline is recorded in `docs/status.md`.
- [x] The engine `KeyHasher` (evaluation.rs) and the facade copy (`edit/diff.rs`) are one definition, or a note says why not.

## Out of scope
The M3 benchmark regression gate.

## Outcome
- `make eval` wall time (single run, same container, warm cache): before T-0930 (`b6aa14e^`) 72.5 s;
  after T-0930 (`main` 44fb924) 71.1 s (-2%, within the 5% budget); this branch 64.6 s (noise-level
  variation; the change here does not touch evaluation).
- `benches/edit.rs`: one warm-up run plus 5 measured runs on fresh workbooks, medians reported.
  Baseline (100k chained formulas, release): recalc 13.4 ms, set untracked 13.6 ms, set tracked
  16.5 ms, diff overhead 3.0 ms, failed-edit rollback 1.8 ms, 1 change.
- `KeyHasher`: one definition in `rustsheet-engine` (`evaluation.rs`, re-exported as
  `rustsheet_engine::{KeyHasher, KeyBuild}`); the facade's copy in `edit/diff.rs` is removed.
