---
id: T-0017
title: "`make new-task` allocates task ids so parallel sessions cannot collide"
milestone: M0
lane: harness
status: todo
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
- [ ] `make new-task MILESTONE=.. LANE=.. TITLE=..` writes a task file from the template with the next free id, computed against `origin/main` as well as the working tree (fetch first).
- [ ] CI's `tasks.py validate` reports a duplicate id with both file paths and the fix (`make new-task` or renumber).
- [ ] Planner, gap-triager and the work-loop skill say to use it.
- [ ] Decision record for the protected-path change; `make check` green; reviewers approve.
