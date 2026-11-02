---
id: T-3759
title: "The task-only fast path counts only task files that are regular files"
milestone: M0
lane: quality
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Only real task files skip the review panel: a PR that edits `docs/tasks/README.md`, or adds a
symlink under `docs/tasks/`, gets the full panel.

## Context
Follow-up from #2736's review (decision 0231, T-3697). `merge_ready.applicable_reviewers` returns
no reviewers when every changed path starts with `docs/tasks/` and ends in `.md`. Two gaps:
- `docs/tasks/README.md` is process documentation, not a task file, yet it qualifies
  (architecture reviewer).
- The test reads names only, so a symlink named `docs/tasks/M9/x.md` (mode 120000) qualifies
  (security reviewer).
Also, two cosmetic nits: the sentence added to SPEC-0027's "Docs-only and task-only PRs" item runs
into the next one on a long line, and the new one-line docstring is dense (quality reviewer).
`scripts/merge_ready.py` is at the 1000-line limit, so a matching helper may belong in
`scripts/tasks.py` (its `TASK_FILE` pattern) rather than here.

## Acceptance criteria
- [ ] A diff qualifies only when every path matches `tasks.TASK_FILE` and is a regular file
      (mode 100644 or 100755) at the head; README.md or a symlink brings the panel back.
- [ ] Unit tests cover README.md alone, a symlink, and a mixed diff.
- [ ] SPEC-0027's item is re-wrapped; decision 0231 says "task files" precisely.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Content checks on task text.
