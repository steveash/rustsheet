---
id: T-1779
title: "Hygiene check: a TODO(T-XXXX) must name an open task"
milestone: M2
lane: quality
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1778]
---

## Goal
`make hygiene` (so `make check` and CI) fails when a `TODO(T-XXXX)` in code names a task that is done,
dropped or missing. A PR that closes a task must then resolve or re-point the TODOs that name it, so
TODOs never end up with no owner again.

## Context
CLAUDE.md allows only `TODO(T-0123)` so every TODO has an owner, but nothing checks that the owner is still
open. On 2026-10-17 six TODOs named done tasks (T-0126, T-0236 three times, T-0931, T-1114), one in the
M2 aggregate walker (`skip_empty_args`); T-1778 resolves them. That is what this check prevents.

Where: `validate` in scripts/tasks.py already runs in `make hygiene` and loads every task. Add a pass
there, or a small `scripts/check_todos.py` called from `hygiene` if it reads better. Scan tracked files
under crates/, bindings/, tools/ and scripts/ (`git ls-files`, so build outputs and .venv are skipped) for
`TODO\(T-(\d{4})\)`. A match is an error unless its task's status is todo, in-progress or blocked. The error
names file:line and the task's status, and says what to do (resolve the TODO, or re-point it to the open
task that owns the work). Leave docs/ out: specs and task files quote old TODOs as history.

scripts/ and the Makefile are protected paths (scripts/check_governance.py), so the PR needs a decision
record (`make new-decision`) that the reviewers approve. It adds a check and weakens nothing.

## Acceptance criteria
- [ ] Unit tests in scripts/test_tasks.py (or a new scripts/test_check_todos.py, found by
      `python3 -m unittest discover -s scripts -p 'test_*.py'`) cover: a TODO naming a todo, in-progress or
      blocked task passes; one naming a done task, a dropped task or an unknown id fails with file:line in the
      message; a bare `TODO` and the task-template placeholder are not matched.
- [ ] On the PR's tree the check passes (T-1778 has landed); with a TODO naming T-0126 added to any .rs file, `make hygiene` fails.
- [ ] A decision record in docs/decisions/ explains the rule and the protected-path change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Resolving existing stale TODOs (T-1778).
- `FIXME` comments, bare TODOs, and TODOs in docs/.
- Checking that the named task is about what the TODO says.
