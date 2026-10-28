---
id: T-2994
title: "Merge-train merge stage: push, CI wait, ordered squash with tree checks, bisect and eject"
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
In `train` mode, `make merge-train` runs one batch from start to finish under the fair lock:
- it pushes the train and waits for CI on the exact sha it built;
- if the train is green, it squash-merges the PRs in order, checking each squash before and
  after;
- if the train is red, it binary-searches the first red prefix and ejects the culprit;
- it recovers from crashes, and always releases the lock.

## Context
- SPEC-0027 section 4, steps 1 and 5–9:
  - the lock: `wait --train`, renewals with exit 4, the exit handler, `HOLD_BUDGET` and
    `CI_BUDGET`;
  - which CI run counts (workflow file, event, branch and sha) and the "green" rule from
    `merge_ready.py`;
  - the mandatory pre-squash check against GitHub's recomputed test merge (`mergeable`,
    `merge_commit_sha`, `GET /git/commits`, the first parent and the tree);
  - the post-squash check, and revert-first plus a P0 on any mismatch;
  - the squash commit text.
- SPEC-0027 section 5 (the guarantee is detective; intermediate states), section 6 (binary search,
  confirmation, infrastructure failures) and section 7 (recovery).

## Acceptance criteria
- [ ] `scripts/merge_train.py` implements the steps above in `train` mode (still unused while
      `mode = "lock"`).
- [ ] `scripts/test_merge_train.py` covers SPEC-0027's Test plan cases for the merge stage:
      - the culprit at `P_5` of 5 (exactly 5 runs);
      - two culprits, an interaction, a failure that does not reproduce, an infrastructure
        failure;
      - `main` moving between squashes, and a merge ref that is never recomputed;
      - lock exit 4, the hold budget, the exit handler;
      - a partly merged batch;
      - the squash text.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Shadow mode, the step-mode fallback and stats (T-2995).
- Turning the mode on (T-2989, T-2991).
