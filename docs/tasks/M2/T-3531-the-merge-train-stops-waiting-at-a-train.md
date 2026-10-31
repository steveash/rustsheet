---
id: T-3531
title: "The merge train stops waiting at a train CI run's first failed job"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A red train CI run costs the train the time to its first failed job, not to its slowest one.

## Context
- The first train after the re-flip to `train` (decision 0208) spent its 40 min hold on four red
  runs while it bisected #2581, each waiting 3-4 min past the eval failure for `rust (test 2/2)`.
  Decision 0212 has the rule.

## Acceptance criteria
- [x] `merge_ready.ci_state(..., early=True)` is RED for an in-progress run once a job of its
      check suite concluded `failure`; PENDING for another suite's failure or a `cancelled` job;
      unchanged without `early` (scripts/test_merge_train_red.py
      `test_a_running_train_ci_is_red_at_its_first_failed_job`).
- [x] The train's CI read passes `early=True`; the scripts tests pass.

## Out of scope
- Cancelling the red run.

## Outcome
Done in this PR.
