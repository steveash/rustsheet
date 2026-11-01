---
id: T-3737
title: "A merger hold that ends with a green train unmerged leaves a record of why"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
When a merger hold ends without merging a train whose CI was green, the reason is recorded where
the next session and the steward can read it (not only in the routine's session transcript).

## Context
- Found by T-3488. On 2026-11-01 the train #2714, #2729, #2727, #2713, #2725 on main b3389661 went
  green at 11:23 UTC (run 41301208007, 6.7 min) but merged nothing; the next hold rebuilt the same
  batch at 11:44 and merged it at 12:05-12:09. No record of why the first hold stopped survives:
  the `merge-lock` ref keeps one commit (no history), the train ref's record commit lists only the
  batch, and the merger's report lives only in its routine session.
- Steward 2026-11-01: again for train #2748, #2676, #2745, #2746, #2750 on main 1f30a9b1. Its
  CI run 41340785823 went green at 19:25 UTC, the hold (`train-1793559067`) still held the lock
  at 19:49 with nothing merged, and the batch squashed only at 20:01-20:04 UTC.
- Candidate causes to tell apart: a step-mode ack timeout (decision 0169), a pre-squash stop, the
  merger session ending (40-min window), or the hold budget.

## Acceptance criteria
- [ ] Every stop of a hold after a green CI wait leaves a durable record: e.g. a `[skip ci]`
      record commit on the train ref naming the stop reason, or a comment on the batch's first PR,
      following the write policy of SPEC-0027 section 4 (step mode where it applies).
- [ ] `make merge-train-stats` (or the steward's daily check) reports green-but-unmerged trains
      with their recorded reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing why holds stop; this task makes the reason visible.
