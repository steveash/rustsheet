---
id: T-3488
title: "Merge train stranded ready PRs: find the cause and meet SPEC-0027's bar before re-flipping to train"
milestone: M2
lane: harness
status: done
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
- [x] A written cause, with the train runs and lock records that show it, for each stranded PR
      listed above (no batch room, lock contention, ejection not labelled, merger not firing).
- [x] The fixes are merged, or filed as tasks this one depends on: T-3476/#2605, #2609 and #2610
      are reviewed against the cause.
- [x] A re-flip to `train` goes through its own gated PR, with a decision record and the
      SPEC-0027 bar (or an owner ruling, decision 0192). Nothing in this task flips it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the stranded threshold itself (that needs a decision record).

## Outcome
**2026-10-31 (the ten listed PRs).** Decision 0208 records the four causes, all found that day:
the merger stopped at step 7.2 on every stale GitHub test merge until #2589 (18:25 UTC; decision
0200), so no train merged anything; a batch took only the 5 oldest ready PRs, so ejections shrank
it (0206); a baseline-only conflict waited for its ended session (0207, #2610); and per-PR merges
took the lock in `train` mode, so the train queued behind them (0204, #2605). Every listed PR met
the first cause; those marked after 15:00 (#2584, #2590) also queued behind the per-PR claim on
#2601. #2590 was the T-3455 duplicate of the merged #2589 and was closed unmerged. The rest
merged: #2550, #2580, #2582, #2583 and #2586 between 19:43 and 22:00 UTC; #2584 at 01:40, #2581
at 07:02, #2437 at 09:15 and #2104 at 09:23 on 2026-11-01. #2609 (fill past ejections) was closed
for #2606's T-3500 version (0206). The re-flip to `train` went through its own PR by owner ruling
(decision 0208, as decision 0192 allows); this task flips nothing.

**2026-11-01 (re-check, 13:30 UTC).** `make merge-train-stats STRANDED=1` listed 21 stranded
PRs (#2738 ... #2668), so the 2026-10-31 fixes were not enough. The lock was free; batches are
built oldest marker first, as 0206 says (#2719, marked 09:26, heads the next batch). The new cause
is the CI timeout: since 2026-10-31 22:50, 31 train CI runs took p50 10.0, p90 10.3 and at most
10.7 min, against `CI_TIMEOUT` = 10 min. A wait past it is an infrastructure failure, retried once
by a push that cancels the running run. Batch #2719, #2716, #2715, #2720, #2691: run 41308888205
was cancelled 18 s after the 13:07:36 retry push, the retry's run 41309736899 ended green at
13:18:21, and the lock was freed at 13:18:08 with nothing merged. Only three batches of five merged
between 09:00 and 13:30 (about 10:08, 11:02 and 12:05 UTC).

**Fix here:** decision 0242 sets `CI_TIMEOUT` to 15 min (scripts/merge_train.py, SPEC-0027
section 4 amended); the unit test reads the constant. **Filed:** T-3736 (a timed-out wait keeps
its running run instead of cancelling it with a retry push), T-3737 (a hold that ends with a green
train unmerged records why: batch #2714 ... #2725 went green at 11:23 and merged only at 12:05,
from the next hold), T-3738 (throughput against arrivals, SPEC-0027's "Revisit"). CI wall time
itself is T-3699 to T-3705. Eval delta: none (harness only).
