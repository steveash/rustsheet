---
id: T-2994
title: "Merge-train merge stage: push, CI wait, ordered squash with tree checks and the revert path"
milestone: M2
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2987, T-2988]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
In `train` mode, `make merge-train TRAIN=1` merges a green batch from start to finish under the
fair lock. It pushes the train, waits for CI on the exact sha it built, and squash-merges the PRs
in order with a check before and after each squash. At every checkpoint it honours the switch.
Any mismatch stops the train with nothing more merged and is reported as a P0. Red trains are
T-2998.

Split per the size clause below (decision 0151): the stop label, the revert PR and the P0
filing moved to **T-3055**.

## Context
- SPEC-0027 section 4: the write policy and steps 1, 5–7 and 9.
  - **Lock and budgets.** `wait --train`, capped at the time left in the window. Renewals with
    exit 4. The exit handler. `HOLD_BUDGET` is counted from acquisition. `CI_TIMEOUT` is 10 min.
    A CI wait needs `CI_TIMEOUT + n × MERGE_BUDGET` left in the budget, where n is the number of
    PRs that run can merge.
  - **Stop checkpoints**, at four points: right after lock acquisition (before the build and
    push), before the CI wait, on every CI poll, and before each squash.
    - Each checkpoint runs `git ls-remote` on `main`, and re-reads
      `origin/main:scripts/merge_train.toml` (failing closed) only when `main`'s sha changed.
    - A `merge-train-stop` label counts only on an open PR that passes `make merge-flip-check`.
      Any other stop label is removed, with a comment.
  - **CI run matching.** Workflow, event `push` or the single fallback `workflow_dispatch`,
    branch and sha. The "green" rule comes from `merge_ready.py`.
  - **Squash checks.** A mandatory pre-squash check against GitHub's recomputed test merge, and a
    post-squash check.
  - **Mismatch response.**
    - Before a squash: stop and file a P0.
    - After a squash: stop merging, push `merge-train/revert-<S_k sha>`, open the revert PR, file
      the P0 in a separate task-only PR, and release. **The revert PR is reviewed and merged under
      the normal flow, with P0 priority.** The merger never merges it, and no review waiver
      applies.
  - **Squash commit text.** Trailers come from structured data. `@`, `#n` and closing keywords in
    titles are neutralised.
- SPEC-0027 section 5 (the guarantee is detective, intermediate states, back-to-back trains) and
  section 7 (recovery).
- With the revert path cut back to "open a PR", this should fit size M. If the diff grows past M,
  split it before review: the push, CI and squash path first, then the checkpoints and the
  revert PR.

## Acceptance criteria
- [x] `scripts/merge_train.py --train` (`make merge-train TRAIN=1`) implements steps 1 and 5-7 of
      SPEC-0027 section 4 for a green train, with the mode half of the stop checkpoints. It does
      nothing (no lock, no write) while `mode = "lock"`.
- [x] `scripts/test_merge_train.py` covers SPEC-0027's Test plan cases for this slice:
      - the write policy in `lock` mode (no lock, no write) and a malformed mode, which reads as `lock`;
      - a mode flip at each checkpoint, including right after acquisition (nothing is pushed);
      - `main` moving before the first squash, between squashes and during a squash, and a merge
        ref that is never recomputed;
      - lock exit 4, the hold budget gate, the capped lock wait and the exit handler;
      - red, missing and overlong train CI;
      - a partly merged batch;
      - the squash text.
- [x] A decision record lands in this PR. The architecture and security reviewers approve.
- Moved to T-3055 (decision 0151): the `merge-train-stop` label at each checkpoint, a stop label
  on a PR that fails `merge-flip-check`, and the revert path (revert PR, P0 filing).

## Out of scope
- Red trains (T-2998).
- Shadow mode, the step-mode fallback and stats (T-2995).
- Turning the mode on (T-2989, T-2991).

## Outcome
`scripts/merge_train.py --train` (`make merge-train TRAIN=1`) runs SPEC-0027 section 4 steps 1
and 5-7 for a green train (decision 0151): the capped lock wait, collect and build with the
train-mode unlabels and ejections, a record commit pushed to `merge-train/current` under a lease,
the hold-budget gate, the CI wait on exactly that sha (one dispatch fallback, 10-min timeout,
renewals every 5 min, `merge_ready.ci_state` with the train's events), then the ordered squash
with a renewal, GitHub's test-merge parent and tree check, `expectedHeadSha`, a lost-reply
read-back and the post-squash parent and tree check. Mode checkpoints at all four points; the
lock is released in a `finally` and on SIGTERM. Inert while `mode = "lock"`.
35 tests in `scripts/test_merge_train.py` (22 new, against a GitHub simulator over a temporary
bare remote). Eval delta: none (scripts and docs only).
Follow-up: **T-3055** (stop label, revert PR, P0 filing); T-2991 now depends on it.
