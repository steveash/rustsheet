---
id: T-3076
title: "make merge-train-stats: merge-train metrics and the shadow agreement score"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-2995]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge train's metrics, and whether shadow predictions agree with what later happened, can be computed from
the GitHub API with one command.

## Context
Split out of T-2995 (its third criterion), which keeps shadow mode.
- SPEC-0027 section 4, "When a shadow prediction agrees": `merge`, `eject:ci-red`, `eject:conflict` and
  `eject:id-collision` are scored against the PR's later state, later heads and their check runs within 24 h;
  `defer` and `no result` are not scored.
- SPEC-0027 "Metrics" and "Daily thresholds": latency per arrival bucket; merge-stage CI runs per merged PR,
  against p, and runner-minutes; lock wait, stranded PRs, the red-train rate, bisection runs and ejections; the
  deferral rate and the effective batch size; tree-check failures and lock occupancy; during shadow, PR CI wall
  time, job queue delay, and T-2991's evidence and load bar (`gh run list --workflow ci.yml --branch
  merge-train/shadow`, plus the commits API).
- Reads go through `scripts/gh_client.py` (T-3001); the shadow commits' message format is T-2995's (decision
  0155). Its `unlabel:` line lists every PR whose label would come off, merges included, not only ejections. After a
  bisection, shadow re-tests the rest of a red train on `M0`, without the green prefix that `train` mode would
  have merged first, so those predictions are approximate.

## Acceptance criteria
- [ ] `make merge-train-stats [SINCE=<date>]` prints every metric above, plus the shadow agreement score and
      the load bar.
- [ ] It is unit-tested against fixtures from the fake client, including PR histories scripted for each
      agreement rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shadow mode itself (T-2995). The step mode (T-3077).
