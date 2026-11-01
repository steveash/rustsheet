---
id: T-3736
title: "A merge-train CI wait that times out keeps the running run instead of cancelling it with a retry push"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
When a merge-train CI wait passes `CI_TIMEOUT` while the run on the tested tip is still in
progress, that run is not thrown away: a retry no longer cancels a run that would have ended green.

## Context
- Found by T-3488 (decision 0242). `Stage.test` in scripts/merge_train.py retries an
  infrastructure outcome by pushing a fresh record commit to `merge-train/current`. CI's
  concurrency group cancels the in-progress run on the old tip. On 2026-11-01 run 41308888205
  (created 12:57:08 UTC, batch #2719, #2716, #2715, #2720, #2691) was cancelled at 13:07:54,
  18 s after the retry push at 13:07:36, when it was about to end; the retry's own run then passed
  the timeout as well and the hold stopped with nothing merged.
- A run that never started, was cancelled or timed out is a real infrastructure failure; a run
  still `in_progress` past the timeout is a slow run, and a second run on the same tree costs at
  least as long again.

## Acceptance criteria
- [ ] A timed-out wait whose run is still in progress either keeps waiting on that run (within
      the hold budget, the gate of SPEC-0027 section 4 step 6 accounting for it) or stops without
      a retry push; it never pushes a record commit that cancels it. Which one is chosen and why
      is in a decision record amending SPEC-0027 section 6.
- [ ] A unit test in scripts/test_merge_train.py covers a run that is still pending at the
      timeout, and the existing "took over N min, twice" case keeps its meaning for runs that
      never started.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The value of `CI_TIMEOUT` (decision 0242).
- Shortening CI itself (T-3699 to T-3705).
