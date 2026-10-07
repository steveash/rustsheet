---
id: T-1122
title: "Fold DATE's inline broadcast into shared NumberOrArray helpers"
milestone: M1
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`fn_date` uses the shared broadcast helpers of `Model::lift_number_pair` (cast.rs) instead of its inline copies.

## Context
T-1119 reviewer: `lift_number_pair` repeats DATE's `any_array`/`broadcast_error` closure, `dims` closure and `elem!` macro (which re-implements `array_node_to_number`), about 60 lines in functions/date_and_time.rs. Extract the error-broadcast step into a shared helper that takes any operand count, or add a 3-ary lift, then delete DATE's copies. Behavior must not change.

## Acceptance criteria
- [ ] DATE's inline broadcast code is gone; existing DATE tests and eval unchanged; duplication ratchet updated with `--update`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added `Model::lift_numbers<N>` (replaces `lift_number_pair`); DATE and MOD use it and DATE's inline broadcast code is gone. No behavior change: eval unchanged (probes 251 -> 251, 0 regressions); duplication ratchet updated with `--update`.
