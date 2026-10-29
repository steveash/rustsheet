---
id: T-3156
title: "Move the merge train's revert and task filing out of merge_train.py"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
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
- [x] The code moves without a behaviour change; `scripts/test_merge_train_stop.py` passes unchanged except for
      import paths.
- [x] `scripts/merge_train.py` is at most 900 lines.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to what the train does.

## Outcome
Moved the mismatch response and task filing (`respond`, `open_revert`, `file_tasks`, `write_task`, `TASK_BODY`,
the prefixes, `Stop`/`Landed`/`CutOff`, `worktrees`, `fetch_main`, `REMOTE`, `MAIN_REF`, `IDENTITY`,
`ID_CHECK_TIMEOUT`) to `scripts/merge_train_filing.py`, which imports nothing from the train (no cycle).
`scripts/merge_train.py` is 1000 -> 840 lines. No behaviour change; tests only changed import and patch
paths. No eval delta.

## Handoff
Pushed on `agent/harness/T-3156-merge-train-revert` (PR #2357). The move is done and `make check` was green locally
(326 script tests), security and quality reviewers approved. CI `hygiene` fails in `check_governance.py`: the diff
changes protected paths (`scripts/merge_train.py`, `scripts/merge_train_filing.py` and three `scripts/test_merge_train*.py`
files) without a decision record in `docs/decisions/`. A sonnet run may not add one (work-loop step 5b). Next: create a
record with `make new-decision TITLE="..."` saying the move is behaviour-neutral, push it, re-run the architecture
reviewer (it asked for changes only over a `ruff format` blank line, already fixed and not a gate), set `status: done`,
and continue at work-loop step 6.
