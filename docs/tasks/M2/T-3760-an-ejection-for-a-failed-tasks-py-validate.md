---
id: T-3760
title: "An ejection for a failed tasks.py validate names the failure instead of an id collision"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
When the merge train ejects a PR because `tasks.py validate` fails on the built train, the
ejection comment says what failed, not "id collision".

## Context
`merge_train.id_collision` names a colliding id only when a failure line matches
`COLLIDING_ID` ("duplicate id" or "decision number"). For any other validate failure it says
"id collision with a task id or decision number; renumber per decision 0121 (2)". On 2026-11-01
#2676 was ejected that way, but merging it with main and running validate showed two cell-ownership
failures: T-3595 listed 8 cells T-3592 already lists, and T-3596 one cell T-3591 lists (parallel
triage PRs). Whoever picked it up would look for an id to renumber that does not exist.

## Acceptance criteria
- [ ] An ejection whose failures include no colliding id names the failure kind and quotes the
      first `FAIL` lines (bounded length), with what to do (fold, drop or name the split task,
      per docs/process/task-format.md).
- [ ] A real id collision still says "id collision with T-XXXX".
- [ ] Unit tests for both, from #2676's failure text.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what validate checks.
