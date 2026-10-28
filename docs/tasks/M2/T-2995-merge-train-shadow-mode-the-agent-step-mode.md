---
id: T-2995
title: "Merge-train shadow mode, the agent step mode fallback and make merge-train-stats"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2994]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge train can be proven live without merging anything. Its metrics can be computed from
the GitHub API. If T-2993 found that `gh api` writes fail in a routine session, the session can
perform the writes safely instead.

## Context
- SPEC-0027 section 4, "Shadow":
  - it pushes to `merge-train/shadow`, takes no lock and merges nothing;
  - its only PR-visible write is the housekeeping for closed PRs;
  - it pushes only when the ready set or `main` changed;
  - each train's commit message records the batch and its predicted outcome;
  - a cancelled or superseded run is recorded as `no result`.
- SPEC-0027 section 4, "GitHub access": the step mode. The script prints one JSON action at a time
  from a fixed schema (`merge`, `label`, `unlabel`, `comment`). The session performs exactly that
  action. The script reads GitHub's state back and aborts on any difference. The session never
  adds, skips, reorders or changes an action. This is built **only if** T-2993's probe found that
  writes fail; otherwise this criterion is dropped, and the PR says so.
- SPEC-0027 "Metrics" and "Daily thresholds": `make merge-train-stats` reports:
  - latency from the first green CI on the final head, bucketed by arrival rate;
  - merge-stage CI runs per merged PR, and runner-minutes;
  - lock wait, stranded PRs, the red-train rate, bisection runs and ejections;
  - the deferral rate and the effective batch size;
  - tree-check failures;
  - lock occupancy;
  - during shadow, PR CI wall time and job queue delay.

  It also supports T-2991's evidence query (`gh run list --workflow ci.yml --branch
  merge-train/shadow`, plus the commits API).

## Acceptance criteria
- [ ] `make merge-train SHADOW=1` implements "Shadow". Tests show it takes no lock and makes no
      PR-visible writes beyond the housekeeping.
- [ ] `make merge-train-stats [SINCE=<date>]` prints every metric above, and prints the shadow
      evidence when it runs in shadow mode. It is unit-tested against fixtures from the fake
      client.
- [ ] The step mode is built and tested, or dropped, depending on T-2993's answer, as stated in
      the PR.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Routine prompts and policy (T-2996). Skills (T-2989).
