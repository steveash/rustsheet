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
- [x] A diff qualifies only when every path matches `tasks.TASK_FILE` and is a regular file
      (mode 100644 or 100755) at the head; README.md or a symlink brings the panel back.
- [x] Unit tests cover README.md alone, a symlink, and a mixed diff.
- [x] SPEC-0027's item is re-wrapped; decision 0231 says "task files" precisely.
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

## Outcome
Done by an opus run resuming the sonnet handoff.
- `Repo.changed_paths` now runs `git diff --raw` and returns each changed path with its mode at the head
  (`000000` for a deletion). `head_state` passes the map through unchanged. A dict iterates as paths, so
  the docs-only, security and excel-fidelity checks are unchanged.
- `review_panel.task_only` reuses `tasks.TASK_FILE` (one definition), so `review_panel` now imports
  `tasks`. A path qualifies only under `docs/tasks/`, matching the pattern, with mode 100644 or 100755.
  Plain names without modes never qualify, which is the safe default.
- What brings the panel back: `docs/tasks/README.md`, a symlink, a submodule, or a deletion of a task
  file. The deletion case is stricter than before, as the criterion says "regular file at the head".
- `merge_ready.py` stays at 1,000 lines (two docstrings tightened).
- Tests: `Applicability.test_only_regular_task_files_skip_the_panel` covers README.md alone, each
  non-regular mode and a mixed diff. `Verify.test_changed_paths_carry_head_modes` runs real git with a
  committed symlink and checks that its marker without verdicts fails V4.
- Docs: SPEC-0027's "Docs-only and task-only PRs" item is re-wrapped, with the train-classification
  sentence split into its own item. Decision 0231 defines "task file". The review, steward and
  work-loop skills use the same wording.
- Eval delta: none (tooling only).

## Landing notes
Paused at the owner's request (2026-11-02, 03:3x UTC). The code, tests and docs are complete and pushed;
draft PR #2833.
- Done: `python3 -m unittest test_merge_ready test_verdict_records` (in scripts/) passes, 64 tests.
- Not done: a full `make check` (stopped while it was compiling), the review panel (design + security,
  because scripts/ changed), the verdict records, `status: done`, and `make mark-ready`.
- Ruff run on scripts/ through the tools/ config reports findings, but main shows the same ones and it is
  not a gate.
