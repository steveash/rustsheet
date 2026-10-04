---
id: T-0932
title: "Record eval run time and a clean edit-bench baseline"
milestone: M1
lane: api
status: in-progress
priority: P3
size: S
depends_on: [T-0930]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Close the one acceptance item T-0930 deferred, and make the edit bench usable as evidence.

## Acceptance criteria
- [ ] `make eval` wall time recorded on `main` before and after the T-0930 changes (no more than 5% regression), noted in this file.
- [ ] `crates/rustsheet/benches/edit.rs` runs several iterations with a warm-up and reports a median; a clean baseline is recorded in `docs/status.md`.
- [ ] The engine `KeyHasher` (evaluation.rs) and the facade copy (`edit/diff.rs`) are one definition, or a note says why not.

## Out of scope
The M3 benchmark regression gate.
