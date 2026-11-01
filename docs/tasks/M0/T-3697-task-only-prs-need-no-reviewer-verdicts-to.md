---
id: T-3697
title: "Task-only PRs need no reviewer verdicts to be marked ready"
milestone: M0
lane: quality
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A PR that only files or edits task files reaches the merge train once CI is green, without the
reviewer panel.

## Context
Owner request, 2026-11-01. `make mark-ready` required three verdicts on every PR, task-only ones
included, and about 10 such PRs sat unmarked for hours. Their real gate is `tasks.py validate`,
which the train runs on every built train.

## Acceptance criteria
- [x] `merge_ready.applicable_reviewers` returns no reviewer for a non-empty diff wholly under
      `docs/tasks/`, and the full panel otherwise; unit tests cover both and a task-only `mark`.
- [x] The review, work-loop and steward skills and SPEC-0027 say so; decision 0231 records it.
- [x] Reviewers approve (verdict records on #2736); CI green before marking.

## Out of scope
- Marking the existing unmarked task-only PRs (any session may do it once this merges).

## Outcome
`applicable_reviewers` returns `()` for a task-only diff, so `mark-ready` and the train's V4 accept
it with no verdicts (decision 0231). Tests: `scripts.test_merge_ready` 42 OK. Only `.md` files under `docs/tasks/` count (security review).
