---
id: T-3155
title: "make mark-ready drops the merge-train-ejected label when it marks a PR again"
milestone: M2
lane: harness
status: done
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

## Outcome
- `scripts/merge_ready.py mark` removes `merge-train-ejected` when it marks a PR. Direct mode deletes the label
  after adding `ready-to-merge`. Step mode prints an `unlabel` action after the `label` action, using the
  existing `merge_step.py` schema (`STEP_UNLABEL` is now shared from merge_ready). Both actions carry the final
  label set, without the ejected label. A PR that is already marked and labelled at its head gets the `unlabel`
  action alone.
- Tests: `test_marking_again_drops_the_ejected_label` covers the step-mode round trip and the unlabel-only case.
  The direct-mode test now expects the DELETE and the final labels, and `test_mark_ready_prints_the_same_schema`
  validates all three actions.
- Decision 0177. The manual-removal sentences are gone from the work-loop (step 6.3) and steward ("Merge train")
  skills.
- Eval delta: none (scripts only).
