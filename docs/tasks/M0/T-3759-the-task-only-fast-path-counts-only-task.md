---
id: T-3759
title: "The task-only fast path counts only task files that are regular files"
milestone: M0
lane: quality
status: in-progress
priority: P2
size: S
tier: opus
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

## Handoff (resumed)
Sonnet run, no code written; handing off as a design question (work-loop step 5b).
- The panel logic now lives in `scripts/review_panel.py` (`task_only`, `applicable_reviewers`), not in
  `merge_ready.py`; `merge_ready` re-exports it. `review_panel.py` says it imports only the standard
  library and `ci_changes`, so using `tasks.TASK_FILE` there needs either an import exception or a shared
  home for the pattern (directive 3: do not copy the regex).
- `applicable_reviewers(paths)` sees names only. A regular-file check needs the head's file modes:
  `Repo.changed_paths` (`merge_ready.py` ~l.471) runs `git diff --name-only`; `--raw -z` would give the
  destination modes (120000 symlink, 160000 submodule). The result flows through `head_state` to the
  two call sites (~l.662 `verify`, ~l.835 `mark`), so the signature or return shape of `changed_paths`
  and `head_state` has to carry the modes.
- `scripts/merge_ready.py` is at exactly 1,000 lines, so the plumbing needs a shrink or a move of code
  into `review_panel.py` or another module first (check `scripts/check_file_size.py`).
- Also needed per the task: re-wrap SPEC-0027's "Docs-only and task-only PRs" item, make decision 0231
  say "task files" precisely (a decision-record edit; check the protected-path rules), and tests in
  `scripts/test_merge_ready.py` for README.md alone, a symlink, and a mixed diff.
- The steward skill (l.73) and the work-loop skill describe a task-only PR as "every changed path a `.md`
  file under `docs/tasks/`"; they may need the same wording fix.
