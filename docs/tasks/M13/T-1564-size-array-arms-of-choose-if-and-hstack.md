---
id: T-1564
title: "Size array arms of CHOOSE, IF and HSTACK/VSTACK from range bounds and copy only picked arms"
milestone: M13
lane: perf
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
CHOOSE over an index array, IF over an array condition and HSTACK/VSTACK hold at most the output plus the arms they actually read: a formula with many large range arms (up to 254 for CHOOSE) is rejected or computed without first copying every arm in full.

## Context
T-1508 performance review: `choose_array` (functions/lookup_and_reference/mod.rs) evaluates every value argument through `evaluate_if_arg` (functions/logical/mod.rs), copying each range into a `Vec<Vec<ArrayNode>>` (each capped by `check_range_size`) before `check_array_size` checks the output, so `CHOOSE({1,2},A1:A1048576,…)` can hold about 254 × 1M nodes. `if_array` and `hstack_vstack.rs` do the same. Excel sizes the result from every arm, picked or not (lookup.md, "CHOOSE over an index array broadcasts its value arguments"), so take dims from `Range` bounds, check the output size, then copy only picked arms (a lazy `IfArg` variant). T-1508 architecture review: while there, move `IfArg`/`if_arg_at`/`if_arg_dims`/`evaluate_if_arg` to a neutral module with a neutral name (e.g. `BroadcastArg`), since CHOOSE shares them.

## Acceptance criteria
- [ ] Measurement in the Outcome: peak memory and time of `CHOOSE({1,2},A1:A1048576,…)` with many unpicked whole-column arms, before/after (release).
- [ ] probes/choose-array-index-range-arms and the IF/HSTACK/VSTACK cases still pass; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to the result values or shapes.
