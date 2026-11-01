---
id: T-3704
title: "CI on pushes to main runs only the eval report"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A push to main runs only what main needs (the informational full eval report and its artifact),
not a second copy of the gates the merge train already ran on the exact merged sha.

## Context
Owner request (2026-11-01, CI-speed investigation). On 2026-11-01 (05:53-09:16Z) 12 of 16 CI runs
on main were cancelled by the next merge's run (concurrency group `ci-${{ github.ref }}`), after
0.2-5.6 min each, so the per-merge eval report of decision 0054 mostly does not exist. In `train`
mode (scripts/merge_train.toml) the train branch's CI tests the sha that lands.

## Acceptance criteria
- [x] A decision record states when main's push run may be trimmed (only when the sha was tested on
      `merge-train/current`, or always) and what keeps the nightly/full eval coverage; it amends
      0054 and 0147.
- [x] CI on main runs the report job without being cancelled by the next merge (e.g. no
      cancel-in-progress for main, or a separate workflow), and skips the duplicated gates if the
      decision allows.
- [x] `scripts/check_ci_workflow.py` and its tests agree; `make check` green; reviewers approve.

## Out of scope
- PR or merge-train CI.

## Outcome
Decision 0239: `cancel-in-progress` is now `${{ github.ref != 'refs/heads/main' }}`.
- A PR's or a train's older run is still cancelled by a newer push.
- A run on main that starts now finishes, with its full gates and decision 0054's eval report.
  On 2026-11-01, 12 of 16 runs on main had been cancelled after 0.2-5.6 min.

The optional trim of main's gates is not taken. In `lock` mode, and for anything outside the
train, the run on main is the only run of the landed sha. Main runs keep every gate.

The effect shows on main's runs after this merges: the share of runs on main that finish, instead
of 4 of 16. The steward's health line reads it.

