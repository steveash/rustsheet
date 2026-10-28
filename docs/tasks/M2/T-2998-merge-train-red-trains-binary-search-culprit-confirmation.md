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
A red train costs at most 1 + ⌈log₂ n⌉ + 1 CI runs. Every PR before the culprit merges, the
culprit is ejected with a clear comment, and an infrastructure failure ejects nobody.

## Context
- SPEC-0027 section 6 is the design:
  - a binary search between the longest known-green prefix and the shortest known-red one;
  - merge `P_1..P_lo` through T-2994's merge stage;
  - one unconditional confirmation run of `P_{lo+1}` on the new `main` (the flake check). If it
    is red, eject the culprit (labels, comment, `merge-train-ejected v1` block, `@` escaped). If
    it is green, file a P1 task and do not eject;
  - re-test the rest as one train;
  - retry an infrastructure failure once;
  - the hold budget;
  - resume state (`known-green-prefix`, `known-red-prefix`, heads) in the train commit message,
    resumed only when `main` and the heads are unchanged.
- The cost bound and the case with the culprit at `P_5` of 5 are in SPEC-0027 section 6.
- If the diff grows past size M, split it before review: the search and the merge of the green
  prefix first, then confirmation, ejection and resume.

## Acceptance criteria
- [ ] `scripts/merge_train.py` implements SPEC-0027 section 6 in `train` mode.
- [ ] `scripts/test_merge_train.py` covers SPEC-0027's binary-search cases, using a scripted CI
      oracle:
      - one culprit at each position, including `P_5` of 5 (exactly 5 runs, `P_1..P_4` merged);
      - two culprits, an interaction, a failure that does not reproduce, an infrastructure
        failure;
      - a bisection cut off by the hold budget, then resumed or restarted.

      In every case, only culprits are ejected, every innocent PR merges, and no untested tree
      lands.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Shadow-mode bisection reporting (T-2995).
