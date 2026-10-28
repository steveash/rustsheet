---
id: T-2990
title: "Measure the merge train for a week and decide whether to retire the per-PR merge path"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2991]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
After seven days in `train` mode, measure the merge train against the 48 h before 2026-10-25.
Then record a decision on whether to retire the per-PR merge path.

## Context
- SPEC-0027 "Metrics" lists each measure with its baseline and target:
  - merges per hour;
  - time from marker to merge, p50 and p90;
  - CI runs per merged PR;
  - lock wait;
  - stranded PRs;
  - ejections and bisection runs;
  - tree-check failures;
  - `main` red after a train;
  - lock occupancy.
- Data sources: the GitHub API (PRs, workflow runs on `merge-train/current` and on PR branches,
  marker and ejection comments) and `main`'s history.

## Acceptance criteria
- [ ] Every metric in SPEC-0027 "Metrics" is reported with the method used to measure it, in the
      task's Outcome or in a research note under docs/research/.
- [ ] Each missed target gets a follow-up task, or a decision record that accepts the miss.
- [ ] A decision is recorded either way: keep `lock` mode as the fallback, or retire it together
      with the per-PR texts. SPEC-0027 is updated to match.

## Out of scope
- Tuning the batch size or the cadence, beyond filing tasks for them.
