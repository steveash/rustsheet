---
id: T-0017
title: "`make new-task` allocates task ids so parallel sessions cannot collide"
milestone: M0
lane: harness
status: done
priority: P0
size: S
depends_on: []
---

## Goal
Two sessions creating tasks at the same time cannot pick the same id.

## Context
2026-10-04: PR #47 created T-0010/T-0011 while main already had different T-0010/T-0011
(duplicate ids fail `scripts/tasks.py validate`). Ids are chosen by hand today.

## Acceptance criteria
- [x] `make new-task MILESTONE=.. LANE=.. TITLE=..` writes a task file from the template with the next free id, computed against `origin/main` as well as the working tree (fetch first).
- [x] CI's `tasks.py validate` reports a duplicate id with both file paths and the fix (`make new-task` or renumber).
- [x] Planner, gap-triager and the work-loop skill say to use it.
- [x] Decision record for the protected-path change; `make check` green; reviewers approve.

## Outcome
`make new-task` / `tasks.py new`: id = next after the highest in the milestone's block, counting
every `origin/*` branch (fetched) plus the working tree; template with fill-in placeholders
that `validate` rejects; duplicate errors name both files. `scripts/test_tasks.py` (7 tests) runs in
`make hygiene`. Planner, gap-triager and task-format.md point at it. Decision 0016. Eval delta: none.
