---
id: T-3476
title: "The merge train fills its batch past ejected candidates"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-3455]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A train's batch holds up to `BATCH_MAX` PRs whenever that many ready PRs merge cleanly. An
ejected or deferred candidate no longer shrinks the batch.

## Context
- Step 2 took only the 5 oldest ready PRs, so ejections inside that window shrank the batch. On
  2026-10-31 a train with 21 PRs ready merged only 2 (#2548, #2568) after ejecting #2585, #2562,
  #2564 and #2565. Decision 0204 has the details.

## Acceptance criteria
- [x] Runs and dry runs build from up to `CANDIDATES_MAX` (20) ready PRs, oldest first; the build
      keeps at most `BATCH_MAX` members.
- [x] Tests: `candidates()` returns 20 of 23 ready PRs in order, and a build with 2 PRs
      conflicting with main still fills to 5 members from the rest.
- [x] SPEC-0027 step 2 and decision 0204 record the change; the scripts tests pass.

## Out of scope
- Resolving ejected PRs' conflicts (baseline-only conflicts are a separate change).

## Outcome
The next live train after this merges should show a full batch whenever 5 candidates merge
cleanly.
