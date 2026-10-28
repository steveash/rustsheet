---
id: T-2995
title: "Merge-train shadow mode, the agent step mode fallback and make merge-train-stats"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2998]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge train can be proven live without merging anything, and its metrics, including whether
shadow predictions agree with reality, can be computed from the GitHub API. If T-2993 found that
`gh api` writes fail in a routine session, the session can perform them safely instead.

## Context
- SPEC-0027 section 4: the write policy, "Shadow" and "When a shadow prediction agrees".
  - Shadow pushes to `merge-train/shadow`, takes no lock and merges nothing. A lease failure on
    the push skips the round.
  - Its only PR-visible write is the closed-PR housekeeping. It pushes only when the ready set or
    `main` changed.
  - Each train's commit message records the batch, the predicted outcome per PR
    (`merge` / `defer` / `eject:<reason>`), the would-be unlabels, and the shadow tree check
    (`tree-check: match | mismatch | n/a`), which compares the first PR's test merge with
    `tree(T_1)`.
  - A cancelled or superseded run is recorded as `no result`.
- The agreement rules are defined in the spec. `merge-train-stats` scores each prediction against
  the PR's later state, its later heads and their check runs, within 24 h.
- The step mode (SPEC-0027 section 4, "GitHub access"):
  - a closed enum of actions, every field generated or validated by the script, a read-back with
    abort on any difference, and a session that never adds, skips, reorders or changes an
    action;
  - T-2986's minimal fallback uses the same schema;
  - built **only if** T-2993's probe found that writes fail. Otherwise this criterion is dropped,
    and the PR says so.
- SPEC-0027 "Metrics" and "Daily thresholds": `make merge-train-stats` reports:
  - latency, per arrival bucket;
  - merge-stage CI runs per merged PR, against p, and runner-minutes;
  - lock wait, stranded PRs, the red-train rate, bisection runs and ejections;
  - the deferral rate and the effective batch size;
  - tree-check failures and lock occupancy;
  - during shadow: PR CI wall time, job queue delay, and T-2991's evidence and load bar (`gh run
    list --workflow ci.yml --branch merge-train/shadow`, plus the commits API).

## Acceptance criteria
- [ ] `make merge-train SHADOW=1` implements "Shadow". Tests show that it takes no lock, makes no
      PR-visible writes beyond the housekeeping, skips a round when the lease fails, and records
      predictions and the tree check.
- [ ] `make merge-train-stats [SINCE=<date>]` prints every metric above, plus the shadow agreement
      score and the load bar. It is unit-tested against fixtures from the fake client, including
      PR histories scripted for each agreement rule.
- [ ] The step mode is built and tested, or dropped, depending on T-2993's answer, as stated in
      the PR.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Routine prompts and policy (T-2996). Skills (T-2989).
