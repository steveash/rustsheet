---
id: T-0034
title: "Parallel functions sub-lanes and multi-task worker sessions"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Function fixes run in parallel and workers spend their sessions working, not starting up: the
functions lane is split by module, lane exclusivity is enforced by the task tool, and both worker
routines loop over tasks for most of each run.

## Context
Owner request (2026-10-05). Function work is the bulk of M2–M10 but ran one task at a time (one
lane). The opus worker stopped after one task and idled for an hour; the sonnet worker stopped
after 3 tasks / 45 min; cold starts are slow. Decision 0022.

## Acceptance criteria
- [x] Nine `functions-*` lanes (agent-workflow.md table); `validate` rejects `lane: functions` on
      open tasks and `new` refuses it; all 34 open functions tasks relabelled by module (33 into
      `functions-*`, T-0920 into `engine`).
- [x] `next`/`ready` skip busy lanes (a pushed agent branch moved in the last 2 h; handoffs and
      branches whose task is done on main excepted; keyed on the task's own lane) and accept `--lane functions` for any functions-* lane; unit tests.
- [x] routines.md: both worker prompts loop until 100 minutes into the session; schedules every
      2 h, staggered; optional second sonnet worker; steward step 2 counts all functions-* lanes.
- [x] work-loop, gap-triager, task-format, Makefile help updated; no `docs/status.md` changes
      (the status-log change is T-0033, another session).
- [x] `make check` green; reviewers approve.

## Out of scope
The status-log contention (T-0033); baseline sharding (conflicts there were rare: 3 merges since
2026-10-01 vs 15 for docs/status.md).

## Outcome
Merged with decision 0022. Owner action: re-apply the routines from docs/process/routines.md
(prompts and schedules changed). Ready after relabelling: 10 sonnet / 22 opus functions tasks
across nine lanes.
