---
id: T-3455
title: "The live merge train checks a stale GitHub test merge locally instead of waiting for it"
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
A live train squash-merges its green members instead of stopping at step 7.2 (the pre-squash
check) because GitHub's test merge of a PR is stale.

## Context
- Every live train on 2026-10-31 stopped at step 7.2, waiting for GitHub to recompute a PR's
  test merge on the current `main`. That happened at 11:51, 13:54, 14:42, 14:55 and 16:19 UTC.
- GitHub recomputes a test merge only when the PR changes. Re-setting the base (tried first in
  this task) did not reliably trigger it. Decision 0200 has the evidence.
- The tests' fake GitHub recomputed on every read, and shadow mode never squashes, so this was
  never exercised before T-2991 turned the train on.

## Acceptance criteria
- [x] `MergeStage.pre_squash` uses GitHub's test merge when it sits on the expected `main`. When
      it does not, `pre_squash` checks that the same merge, done locally with `git merge-tree`,
      gives the train's tree. A different tree is a P0, as is a moved `main`; a conflict stops.
- [x] The post-squash check and its revert path are unchanged.
- [x] The fake GitHub keeps stale test merges. The tests cover a stale and a computing test
      merge, a local tree mismatch (P0) and a local conflict.
- [x] SPEC-0027 step 7.2 and its safety argument, and decision 0200, record the change.
- [x] The scripts tests pass.

## Out of scope
- Making GitHub recompute test merges.
- Per-PR merges by sessions still on the lock flow. They hold the lock, so they are serialised
  with the train.

## Outcome
The first live train after this merges is the acceptance run: its log should show "GitHub's test
merge is stale; its merge onto ... has the train's tree", followed by the squash.
