---
id: T-2994
title: "Merge-train merge stage: push, CI wait, ordered squash with tree checks and the revert path"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2987, T-2988]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
In `train` mode, `make merge-train` merges a green batch from start to finish under the fair lock:
- it pushes the train and waits for CI on the exact sha it built;
- it squash-merges the PRs in order, with a check before and after each squash;
- it honours the switch and the stop label at every checkpoint;
- it reverts a mismatching squash itself.

Red trains are handled by T-2998.

## Context
- SPEC-0027 section 4, the write policy and steps 1 and 5–7, 9:
  - the lock: `wait --train`, capped at the window's remaining time; renewals with exit 4; the
    exit handler; `HOLD_BUDGET` counted from acquisition; no CI wait unless `CI_TIMEOUT + n ×
    MERGE_BUDGET` remains;
  - stop checkpoints: re-fetch `origin/main:scripts/merge_train.toml` (failing closed) and look
    for the `merge-train-stop` label before the CI wait, on every CI poll and before each squash;
  - CI-run matching: workflow, event `push` or the one `workflow_dispatch`, branch, sha. The
    "green" rule comes from `merge_ready.py`;
  - the mandatory pre-squash check against GitHub's recomputed test merge, and the post-squash
    check;
  - the response to a mismatch. Before the squash: stop and file a P0. After it, the revert path
    the merger owns: `merge-train/revert-<sha>`, a revert PR, the P0 in a separate task-only PR,
    and `make merge-revert-check`, then a merge with `expectedHeadSha` under the lock, or a
    hand-off to the P0 handler;
  - the squash commit text: trailers from structured data; titles with `@`, `#n` and closing
    keywords neutralised.
- SPEC-0027 section 5 (the guarantee is detective; intermediate states; back-to-back trains) and
  section 7 (recovery).
- If the diff grows past size M, split it before review: the push, CI and squash path first, then
  the revert path.

## Acceptance criteria
- [ ] `scripts/merge_train.py` implements the above in `train` mode. It stays unused while
      `mode = "lock"`.
- [ ] `make merge-revert-check PR=<n>` accepts exactly a single-commit inverse of the named squash
      on the current `main`, and refuses anything else (tested).
- [ ] `scripts/test_merge_train.py` covers SPEC-0027's Test plan cases for the merge stage:
      - the write policy;
      - a mode flip or stop label mid-batch, at each checkpoint;
      - a malformed mode, which reads as `lock`;
      - `main` moving between squashes;
      - a merge ref that is never recomputed;
      - the revert path;
      - lock exit 4, the hold budget, the capped lock wait and the exit handler;
      - a partly merged batch;
      - the squash text.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Red trains (T-2998). Shadow mode, the step-mode fallback and stats (T-2995).
- Turning the mode on (T-2989, T-2991).
