---
id: T-2998
title: "Merge-train red trains: binary search, culprit confirmation, ejection and infrastructure retry"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2994]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A red train costs at most 1 + ⌈log₂ n⌉ + 1 CI runs, and the bound holds across lock holds. Every
PR before the culprit merges, and the culprit is ejected with a clear comment. Every hold that
ran a green CI makes progress, and an infrastructure failure ejects nobody.

## Context
- SPEC-0027 section 6 is the design:
  - a binary search between the longest known-green prefix and the shortest known-red one;
  - merge `P_1..P_lo` through T-2994's merge stage;
  - one unconditional confirmation run of `P_{lo+1}` on the new `main`, which is the flake check.
    - Red: eject the culprit (labels, a comment, the `merge-train-ejected v1` block, `@`
      escaped).
    - Green: file a P1 task and do not eject;
  - re-test the rest as one train;
  - retry an infrastructure failure once.
- SPEC-0027 section 4, step 6, defines the budget gate. A CI wait needs `CI_TIMEOUT` (10 min) plus
  n × `MERGE_BUDGET` left in the hold budget, where n is the number of PRs that run can merge:
  `mid` for a bisection step, 1 for the confirmation run, the batch size for a full train.
- On a cut-off, the merger merges the known-green prefix before it releases. A gate failure
  before the confirmation run is a cut-off too: it records `known-red: P_{lo+1}` so the resumed
  hold goes straight to the confirmation (otherwise a culprit at `P_1` or `P_2` livelocks near
  the 10-min timeout).
- **Resume is required.** The cut-off writes `bisect-base`, `known-red` (with the heads) and
  `known-green-prefix` into a commit on `merge-train/current`. The next pass resumes from that
  state when `main` is still `bisect-base` and the heads are unchanged. In `train` mode only the
  merger moves `main`, so resume is the normal path.
- SPEC-0027 section 6 works through an example, the culprit at `P_5` of 5: one hold of about 31
  min with typical CI, or two holds when every run hits the 10-min timeout. Either way it takes 5
  runs and no third rebuild.
- If the diff grows past size M, split it before review: the search, the gate and merge-on-cut-off
  first, then confirmation, ejection and resume.

## Acceptance criteria
- [ ] `scripts/merge_train.py` implements SPEC-0027 section 6 in `train` mode, with the per-wait
      gate, merge-on-cut-off and required resume.
- [ ] `scripts/test_merge_train.py` covers SPEC-0027's binary-search cases with a scripted CI
      oracle:
      - one culprit at each position, including `P_5` of 5 (exactly 5 runs, `P_1..P_4` merged);
      - **a 5-PR red train with the culprit at `P_4`, and again at `P_5`, with CI at the 10-min
        timeout.** It is cut off, the known-green prefix merges before the release, and resume
        completes the train in the next hold. At most 5 runs, no third rebuild;
      - **the culprit at `P_1`, and again at `P_2`, of 5 with CI at the 10-min timeout.** The
        confirmation gate fails, the cut-off records the single known-red PR, and the next hold
        confirms and ejects it without re-running the full train. At most 5 runs;
      - a restart, with its reason recorded, when `main` or a head changed;
      - two culprits, an interaction, a failure that does not reproduce, and an infrastructure
        failure.

      In every case only culprits are ejected, every innocent PR merges, and no untested tree
      lands.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Shadow-mode predictions. T-2995 reuses this search for them.
