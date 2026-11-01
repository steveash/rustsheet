---
id: T-3539
title: "A new-coverage rise of the visible-parity lock keeps main's slack"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A probe PR that adds visibly-off cells records a lock that stays valid when `main` gains cells
inside its own slack, so the PR is not ejected from the merge train for a count it did not change.

## Context
- Decision 0093: `check` fails only a rise above `eval/baseline/visible-parity.json`; a count below
  the lock is slack the steward takes back daily with `--lower-visible-lock`.
- `visible_parity::judge` records a new-coverage rise, and `visible_parity_gate`
  (crates/rustsheet-eval/src/main.rs) then writes the exact count. That throws main's slack away
  on the PR branch: the PR's lock is `main's count + new cells`, not `main's lock + new cells`.
- Observed on 2026-10-31 with #2581 (T-3298, 130 new visibly-off cells): main's lock was 2427 with
  slack; the PR recorded 2556, then 2542, 2544, 2548, 2561 and 2563 as main merged PRs whose new or
  newly visibly-off cells fit main's slack but not the PR's exact lock. The train ejected it
  (`ci-red`, visible parity) and every merge of `main` re-ran the re-record.

## Acceptance criteria
- [x] A new-coverage rise records `lock + new_coverage` (main's slack kept), not the run's count;
      `--allow-regressions` and `--lower-visible-lock` keep recording the count.
- [x] Unit tests in visible_parity.rs pin the recorded value for each path.
- [x] Decision record amending 0093 (0214).
- [x] `make check` green; reviewers approve.

## Out of scope
- How the steward lowers the lock.

## Outcome
- `visible_parity::judge` returns `Judgement::Record(lock)`. A new-coverage rise records the old
  lock plus the new cases' visibly-off cells. A drop (`--lower-visible-lock`) and a rise beyond the
  new coverage under `--allow-regressions` record the count. `visible_parity_gate` writes the value.
  The unit test `a_new_coverage_rise_keeps_the_slack` pins it. Decision 0214 amends 0093.
- Eval delta: none (recording logic only; no case, baseline or lock change in this PR).
