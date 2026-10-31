---
id: T-3076
title: "make merge-train-stats: merge-train metrics and the shadow agreement score"
milestone: M2
lane: harness
status: done
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
- [x] `make merge-train-stats [SINCE=<date>]` prints every metric above, plus the shadow agreement score and
      the load bar.
- [x] It is unit-tested against fixtures from the fake client, including PR histories scripted for each
      agreement rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- Shadow mode itself (T-2995). The step mode (T-3077).

## Outcome
- `scripts/merge_train_stats.py`, `make merge-train-stats [SINCE=<date>]` (decision 0167). It prints merges per
  hour, latency p50/p90 per arrival bucket, merge-stage CI runs and runner-minutes per merged PR, PR CI wall time
  and job queue delay, lock holds, wait p90 and held share, stranded PRs, red-train rate and bisection runs per
  train branch, red `main` runs, shadow ejections by reason, deferral rate, effective batch size and tree-check
  failures, the steward's daily thresholds, the shadow agreement score (with every disagreement listed) and
  T-2991's evidence and load bar.
- Shadow predictions are `[skip ci]` commits with no run, so they come from the activity API of
  `merge-train/shadow`; the SPEC-0027 evidence paragraph now says so. `gh_client` gains `activity` (cursor
  paging), `run_jobs`, `updated_pulls` and `event`/`created` run filters.
- Tests: `scripts/test_merge_train_stats.py` (each agreement rule on scripted PR histories, prediction parsing, lock
  holds and waits, train grouping, a whole report from the fake client); `test_gh_client.py` covers the new reads
  and the cursor.
- Live run on 2026-10-29 20:30-22:18 UTC (1 min 38 s): 7 merged PRs, latency p90 4.0/4.5 min, lock held 97% of the
  time with wait p90 51 min, #2344 stranded, no shadow prediction yet (`merge-train/shadow` has one push, from
  2026-10-28), so the T-2991 bar is not met.
- Train-mode-only sources (ejections, deferrals and tree checks on `merge-train/current`, which records no
  predictions) are left to T-2990, which measures train mode.
- Eval delta: none (scripts and docs only).
