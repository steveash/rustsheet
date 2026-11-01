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
After seven days in `train` mode, measure the merge train against the 48 h before 2026-10-25 and
record whether to retire the per-PR merge path.

## Context
- SPEC-0027 "Metrics" gives the measures, their baselines and their targets:
  - latency, measured from the first green CI on the final head and bucketed by arrival rate, with
    the p90 target conditional on the measured breakage rate p;
  - merge-stage CI runs per merged PR, and runner-minutes;
  - lock wait, and stranded PRs;
  - the red-train rate against 1 − (1 − p)^n, bisection runs and ejections;
  - the deferral rate and the effective batch size;
  - tree-check failures, and `main` red after a train;
  - lock occupancy.
- `make merge-train-stats SINCE=<date>` (T-2995) computes them.
- This task also checks the first live trains after T-2991's switch: whether they merged their
  batches with every tree check passing.

## Acceptance criteria
- [ ] Every metric in SPEC-0027 "Metrics" is reported with its method, in the task's Outcome or in
      a research note under docs/research/.
- [ ] The first live trains after the switch are listed with their outcomes.
- [ ] Each missed target gets a follow-up task, or a decision record that accepts the miss.
- [ ] A decision is recorded: keep `lock` mode as the fallback, or retire it along with the per-PR
      texts. SPEC-0027 is updated to match.

## Out of scope
- Tuning the batch size or the cadence, beyond filing tasks for them.
