---
id: T-3515
title: "A per-PR merge-lock wait stops when the mode switches to train"
milestone: M2
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A session waiting for the merge lock for its own PR stops waiting, and leaves the queue, as soon
as `mode` switches to `train`, so it cannot take the lock ahead of a train.

## Context
- Decision 0204 (#2605) refuses a per-PR claim in `train` mode, but `wait` read the mode only
  when it started. On 2026-10-31, after the re-flip to `train` (#2623, decision 0208), the
  waiters that had queued under `lock` kept polling; one of them (#2624) took the lock the moment
  it was freed, ahead of the train. Decision 0209 has the rule.

## Acceptance criteria
- [x] `merge_lock.wait` asks a `refusal` callback before every poll; a reason ends the wait with
      exit 5 and removes the waiter from the queue (`merge_lock.leave`). `main` passes
      `train_mode_refusal` for `--pr` waits, nothing for `--train`
      (scripts/test_merge_lock.py `test_a_wait_that_began_before_a_switch_to_train_stops_and_leaves_the_queue`).
- [x] The scripts tests pass.

## Out of scope
- A per-PR holder that already has the lock when the mode switches: it finishes its merge.

## Outcome
Done in this PR. Waits now re-read the mode every 30 s poll, one `git fetch` of `main` each.
