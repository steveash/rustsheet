---
id: T-3156
title: "Move the merge train's revert and task filing out of merge_train.py"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-3055]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
`scripts/merge_train.py` has room under the 1000-line file-size ratchet again, so the next merge-train task does
not have to split it first.

## Context
T-3055 left `scripts/merge_train.py` at exactly 1000 lines (`scripts/check_file_size.py`'s limit) after moving
shadow mode to `scripts/merge_train_shadow.py`. The architecture reviewer suggested the next seam: the
mismatch response and task filing (`Stage.respond`, `open_revert`, `file_tasks`, `write_task`, `TASK_BODY`,
`REVERT_PREFIX`, `TASKS_PREFIX`, `TASK_MILESTONE`, `TASK_LANE`) as functions in their own module (for example
`scripts/merge_train_filing.py`) taking the root, the client and the log. `worktrees`, `fetch_main`, `IDENTITY`
and `REMOTE` stay in `merge_train.py` or move to a module both import; avoid an import cycle.

## Acceptance criteria
- [ ] The code moves without a behaviour change; `scripts/test_merge_train_stop.py` passes unchanged except for
      import paths.
- [ ] `scripts/merge_train.py` is at most 900 lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what the train does.

## Outcome
Moved the mismatch response and task filing (`respond`, `open_revert`, `file_tasks`, `write_task`, `TASK_BODY`,
the prefixes, `Stop`/`Landed`/`CutOff`, `worktrees`, `fetch_main`, `REMOTE`, `MAIN_REF`, `IDENTITY`,
`ID_CHECK_TIMEOUT`) to `scripts/merge_train_filing.py`, which imports nothing from the train (no cycle).
`scripts/merge_train.py` is 1000 -> 843 lines. No behaviour change; tests only changed import and patch
paths. No eval delta.
