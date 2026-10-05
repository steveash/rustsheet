---
id: T-0037
title: "Worker budget: 110 minutes, 10 tasks"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Each worker run, opus as well as sonnet, keeps taking tasks for almost all of its 2-hour slot.

## Context
Owner request (2026-10-05): raise the budget from 100 minutes / 6 tasks (decision 0022) to 110
minutes / 10 tasks, and make sure the opus worker loops too. The work-loop skill still said the
opus worker completes "exactly one task per run". Decision 0025.

## Acceptance criteria
- [x] Both worker prompts in docs/process/routines.md say 110 minutes / 10 tasks; the
  agent-workflow note and decision 0022 point to decision 0025.
- [x] The work-loop skill loops back to step 1 after a merge, and a blocked or handed-off task
  ends work on that task only.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying the routines (the owner does that; agents never modify routines).

## Outcome
Budget raised to 110 min / 10 tasks in both prompts; the stale opus "one task per run" line is
removed from the work-loop skill. The prompts take effect when the owner re-applies them.
