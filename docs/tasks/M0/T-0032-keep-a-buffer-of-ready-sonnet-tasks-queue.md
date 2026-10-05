---
id: T-0032
title: "Keep a buffer of ready sonnet tasks (queue health)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The sonnet worker never idles for lack of work: at least 10 `tier: sonnet` tasks are ready and
unclaimed at all times, refilled by the runs that can plan (steward, opus worker).

## Context
The sonnet worker runs hourly and takes up to 3 tasks, but only the opus worker (when it finds no
opus task) and the daily steward (when a lane has fewer than 3 / 2 ready tasks of any tier) run
the gap-triager or planner. Nothing counts ready *sonnet* tasks, so the cheap tier can drain its
queue and stop while opus work piles up (2026-10-05: 14 ready sonnet vs 45 ready opus).
Decision 0021.

## Acceptance criteria
- [x] `python3 scripts/tasks.py ready [--lane L] [--tier T]` lists every task `next` could pick
      (same filter: ready, tier incl. handoffs, unclaimed) and ends with `N ready`; `next` and
      `ready` share one filter (`available`); unit test in scripts/test_tasks.py.
- [x] The opus worker (work-loop) and the steward (agent-workflow.md, which its routine reads)
      top the sonnet queue up to the buffer; the gap-triager and planner cut sonnet-sized tasks
      when refilling, without mislabelling tiers. Routine prompts unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
Splitting the functions lane, sharding baselines, changing routine schedules (owner decision).

## Outcome
`scripts/tasks.py ready`, decision 0021, work-loop step 6.5, agent-workflow queue-health rule,
gap-triager/planner refill guidance. Buffer at merge time: 14 ready sonnet tasks.
