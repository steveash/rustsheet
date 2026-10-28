---
id: T-2991
title: "Turn the merge train on after a day of shadow runs by the merger routine"
milestone: M2
lane: harness
status: blocked
priority: P1
size: S
tier: opus
depends_on: [T-2989, T-3045]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge train goes live. The switch moves to `mode = "train"` after at least a day of shadow
runs by the owner's merger routine, provided the shadow evidence meets SPEC-0027's bar.

## Context
- SPEC-0027 Rollout, "Shadow evidence for T-2991". The evidence comes from the GitHub API, not
  from branch history, because the branch is force-pushed:
  - `gh run list --workflow ci.yml --branch merge-train/shadow --created ">=<date>" --json
    headSha,conclusion,createdAt` lists the runs;
  - the commits API reads each `headSha`, whose message holds the batch, the per-PR predictions
    and the `tree-check` record;
  - `make merge-train-stats` (T-2995) computes the score.
- The bar (SPEC-0027 Rollout):
  - at least 10 scored shadow trains over at least 24 h, the newest within the last 2 h;
  - at least 90% of scored predictions agree, by SPEC-0027 section 4's rules, and every
    disagreement is explained;
  - zero `tree-check: mismatch` records;
  - load: PR CI p90 wall time no more than 20% above the 48 h before shadow started, and job queue
    delay p90 under 1 min;
  - the red-train rate is reported against p = 0.09.

## Blocked because
This needs the owner's merger routine (SPEC-0027 "Owner steps"), in addition to the dependency on
T-2989. To check whether the blocker is gone (decision 0013), run
`gh run list --workflow ci.yml --branch merge-train/shadow --limit 50 --json createdAt`. It must
show runs spanning at least 24 h, with one in the last 2 h. Until it does, the routine is missing
or shadow mode has not run long enough.

## Acceptance criteria
- [ ] The PR gives `make merge-train-stats`'s shadow evidence against every point of the bar,
      with run links.
- [ ] `scripts/merge_train.toml` says `mode = "train"`. A short decision record cites the evidence
      and T-2996's record.
- [ ] The Outcome links any live trains that ran during this session. The first live trains are
      checked by T-2990 at its checkpoint, so this task does not wait for them. A tree-check
      failure at any time means the emergency flip back to `lock`, with a P0 task.

## Out of scope
- The metric review (T-2990).
