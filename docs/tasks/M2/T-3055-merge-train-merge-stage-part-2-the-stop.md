---
id: T-3055
title: "Merge-train merge stage, part 2: the stop label and the revert PR"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-2994, T-2996]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A train in `train` mode honours a valid `merge-train-stop` label at every stop checkpoint, and a
post-squash mismatch ends with a revert PR (never merged by the merger) and a filed P0.

## Context
T-2994 split along the seam its task file named (decision 0151): it shipped the push, the CI
wait and the ordered squash with the pre- and post-squash checks, and the mode half of the stop
checkpoints (`Stage.checkpoint` in `scripts/merge_train.py`). A mismatch there is only reported
(`P0: ...` in the log, exit 1). This task adds the rest of SPEC-0027 section 4:
- **Stop label.** At each checkpoint, list open PRs labelled `merge-train-stop`; honour one that
  passes `make merge-flip-check` (T-2996), and remove the label with a comment
  (`stop label removed: not a valid flip PR`) from any other PR.
- **Revert path (step 7, "Response to a mismatch").** After a squash mismatch: on a scratch
  worktree at the current `origin/main`, `git revert --no-edit S_k`, push
  `merge-train/revert-<S_k sha>`, open the revert PR (script-generated body naming `S_k`, the
  expected and the actual tree), file the P0 task in a separate task-only PR naming it, release.
  Before a squash: file the P0 only. `gh_client` needs a create-PR call; check T-3001's probe on
  which writes a session may make (the step mode of T-2995 may be the path).
- **The P1 of a red train that did not reproduce** (T-2998, decision 0154): `Stage.tasks`
  holds it as (priority, title, body); file it with the same task-only PR path as the P0.
- **Branch deletion** after a merge (step 7.5), ignoring the proxy's 403 (T-1938).
- `make merge-mode` prints `train (stop pending: #n)` when a stop label is honoured.

## Acceptance criteria
- [ ] `scripts/test_merge_train.py`: a valid stop label at each checkpoint (right after
      acquisition: nothing pushed), a label on a PR that fails `merge-flip-check` is ignored and
      removed with a comment, and the revert path (PR opened, P0 filed, lock released, the revert
      PR never merged by the merger).
- [ ] `make check` green; architecture and security reviewers approve.

## Out of scope
- Red trains (T-2998), shadow mode and the step mode (T-2995), turning the mode on (T-2989, T-2991).
