---
id: T-3488
title: "Merge train stranded ready PRs: find the cause and meet SPEC-0027's bar before re-flipping to train"
milestone: M2
lane: harness
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Find why PRs marked ready sat unmerged for more than 2 h in the first live `train` day. Fix the
cause, or file the fix, so the "stranded" daily threshold of SPEC-0027 is no longer met. Until
then the mode stays `lock`.

## Context
- The 2026-10-31 18:43 UTC steward run found 10 stranded PRs: their newest merge-ready marker was
  more than 2 h old, they were unmerged, and none was ejected. This is the `make merge-train-stats
  STRANDED=1` list: #2590, #2586, #2584, #2583, #2582, #2581, #2580, #2550, #2437 and #2104.
  Examples: #2580 was marked at 14:48 UTC, #2584 at 15:03 and #2590 at 15:37. At 18:45 UTC all
  were still open and labelled `ready-to-merge`.
- Under the `steward` skill ("Merge-train daily check"), a stranded PR in `train` mode triggers
  the emergency flip to `lock` (decision 0157). The flip PR is #2613, labelled `merge-train-stop`.
- Observed facts, to explain:
  - `merge-train/current` had 11 CI runs from 12:05 to 17:35 UTC, all green. There was then no
    train run for over an hour, while 20 or more PRs were ready.
  - In that window, 35 commits reached `main` after #2523, and some trains merged only one or two
    PRs. Is the batch filled? (#2609, T-3476 "fills its batch past ejected candidates", and #2610,
    T-3487, are open on this.)
  - At 18:45 UTC the merge lock was held for a per-PR claim (PR 2601, 17 min), with a train
    queued behind it. T-3476 (#2605) makes the lock refuse per-PR claims in `train` mode.
  - #2590 and the merged #2589 both carry task id T-3455 (a duplicate to close per the `steward`
    skill).
- Decision 0192: going back to `train` meets SPEC-0027's full bar again, or needs its own owner
  ruling.

## Acceptance criteria
- [ ] A written cause, with the train runs and lock records that show it, for each stranded PR
      listed above (no batch room, lock contention, ejection not labelled, merger not firing).
- [ ] The fixes are merged, or filed as tasks this one depends on: T-3476/#2605, #2609 and #2610
      are reviewed against the cause.
- [ ] A re-flip to `train` goes through its own gated PR, with a decision record and the
      SPEC-0027 bar (or an owner ruling, decision 0192). Nothing in this task flips it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the stranded threshold itself (that needs a decision record).
