---
id: T-3280
title: "ifs_cache: one resumable inert-check-then-read helper for ifs_range_values and gathered_values"
milestone: M11
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ifs_cache.rs` checks that a set of cells can be read without evaluating anything, resuming where it stopped, then
reads them, in one helper, so `ifs_range_values` and `gathered_values` cannot drift apart.

## Context
Quality review of T-0498 (PR #2442): `Model::gathered_values` repeats the tail of `Model::ifs_range_values` in
crates/rustsheet-engine/src/functions/statistical/ifs_cache.rs: the resumable `read_is_inert` loop with its
`RangeState::Waiting`/`Never` transitions, the `MAX_CACHED_CELLS` check, the `evaluate_cell` reads and the second
`pass_cache_safe` check. Only the cell indexes differ (`0..size` against `set_bits(matched)`). A pure refactor: no
behavior change.

## Acceptance criteria
- [ ] One helper, taking the cell indexes as an iterator and the map entry to update, serves both callers.
- [ ] `test_ifs_cache.rs` passes unchanged; `scripts/check_duplication.py` does not grow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what is cached or when.
