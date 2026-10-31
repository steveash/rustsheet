---
id: T-3455
title: "The live merge train re-sets a stale PR's base so GitHub recomputes its test merge"
milestone: M2
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A live train squash-merges its green members instead of stopping at step 7.2, the check that each
PR's GitHub test merge sits on the current `main` with the train's tree.

## Context
- The first live train (2026-10-31, `merge-train/current` run 41198551221) tested #2533, #2524
  and #2546 green, then stopped: "GitHub's test merge of #2533 was not recomputed on 4ff408243c5e
  within 2 min".
- GitHub recomputes a test merge only when the PR changes, so every open PR's test merge was on an
  old `main`. Re-setting the base to `main` (`PATCH pulls/{n}`) made GitHub recompute it. Decision
  0200 has the evidence.
- The tests' fake GitHub recomputed on every read, and shadow mode never squashes, so this was
  never exercised before T-2991 turned the train on.

## Acceptance criteria
- [x] `MergeStage.pre_squash` re-sets a stale PR's base once (`Client.set_base`) and polls for up
      to 5 minutes. A tree mismatch or a moved `main` still stops the train as before.
- [x] Step mode has the `set_base` action (`pr`, `base` = `main`), validated and read back
      (`base.sha` = the tip of `main`). SPEC-0027 and the merger prompt in routines.md list it.
- [x] The fake GitHub keeps stale test merges until the base is re-set.
      `test_a_stale_test_merge_is_recomputed_by_re_setting_the_base` and the step-mode tests cover
      it.
- [x] The scripts tests pass; decision 0200 records the change.

## Out of scope
- Per-PR merges by sessions still on the lock flow, which keep moving `main` between trains.
  They hold the lock, so they are serialised with the train.

## Outcome
Live evidence and the fix are in decision 0200. The first live train after this merges is the
acceptance run: it should squash its members, re-setting the base of each stale one.
