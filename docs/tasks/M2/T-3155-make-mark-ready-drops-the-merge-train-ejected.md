---
id: T-3155
title: "make mark-ready drops the merge-train-ejected label when it marks a PR again"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2989]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A PR that `make mark-ready` marks again after an ejection no longer carries the `merge-train-ejected`
label, with no manual step.

## Context
- SPEC-0027 section 6, "After an ejection": the label `merge-train-ejected` "is removed when the PR is
  marked again". `scripts/merge_ready.py`'s `mark` keeps every existing label and only adds
  `ready-to-merge` (`labels` of the step-mode `label` action is the old set plus it).
- T-2989 tells workers to remove the label by hand after marking (work-loop step 6.3, steward "Merge
  train"); this task makes `mark` do it, then drops that sentence from both skills.

## Acceptance criteria
- [ ] Direct mode removes `merge-train-ejected` after adding `ready-to-merge`; step mode's `label` action's
      `labels` set omits it. Unit tests in `scripts/test_merge_ready.py` cover both.
- [ ] The manual-removal sentence is gone from the work-loop and steward skills (protected path:
      decision record and architecture/security approval).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any other label or the merger's own writes.
