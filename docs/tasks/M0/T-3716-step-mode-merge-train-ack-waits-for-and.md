---
id: T-3716
title: "Step mode: merge-train-ack waits for and prints the next action, and merge-train-next blocks for one, so the merger stops sleeping and missing actions"
milestone: M0
lane: harness
status: done
priority: P1
size: S
depends_on: []
---

## Goal
A step-mode merger session performs each GitHub write as soon as the run asks for it, instead of
sleeping a fixed time between writes and sometimes missing an action.

## Context
- The owner asked on 2026-11-01 why nothing merged from 07:10 to 09:05 UTC. What the CI and lock
  records show:
  - No train run between 07:01 and 07:54.
  - The 07:54 train run's jobs all finished by 08:04, but GitHub recorded its last job
    (`rust (test 2/2)`, final step at 08:04:15) as completed only at 09:04:17. One hold waited the
    whole hour on that tip and merged it at 09:05 (comment on #2594: "merged by merge train
    864763e9e"). That is beyond `CI_TIMEOUT` (10 min) and `HOLD_BUDGET` (40 min), which the train
    measures with `time.monotonic`. The likely reading is that the merger session's VM was paused
    during the wait, and paused time doesn't advance the monotonic clock. This is unverified: the
    07:35 merger run's transcript isn't reachable from a later session.
  - The merger routine runs hourly (`35 * * * *`), while docs/process/routines.md says every 30
    minutes (`5,35 * * * *`). With a 40-minute window, that leaves about 20 minutes an hour with no
    merger. The routine is the owner's to change.
- What this task fixes: the 09:35 merger run's transcript shows the step-mode loop spending about
  45 s per GitHub write (ack, `sleep 40`, grep the log), about 11 minutes per 5-PR batch. It also
  missed an eject action printed at 09:57 for 10 minutes. Decision 0238.

## Acceptance criteria
- [x] `make merge-train-ack` waits up to `NEXT`=90 s for the run's next action and prints it.
- [x] `make merge-train-next` blocks up to `WAIT`=300 s for a pending action and prints it (exit 3:
      none).
- [x] The run's protocol is unchanged (ack file, `ACK_TIMEOUT`, read-backs); existing step tests pass.
- [x] New unit tests cover `wait_next`, `ack --next` and `next`.
- [x] docs/process/routines.md's merger prompt uses both commands; decision 0238 records the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- The merger routine's schedule and prompt in the routine itself (owner-only).
- A guard for a hold whose session is paused: it needs evidence from a merger transcript first.

## Outcome
`merge_step.py` gains `wait_next`, `ack --next S` and `next --wait S`; the Makefile's
`merge-train-ack` passes `--next 90`, and a new `merge-train-next` target waits for an action.
The step tests pass: 20, 5 of them new. `next` never prints an action whose ack is still on disk
(review round 2), and the waits share `merge_lock.non_negative` with the lock's `--timeout`. The owner should update the live merger routine's prompt
from docs/process/routines.md, and decide its schedule: it runs hourly, but the doc says every
30 minutes.
