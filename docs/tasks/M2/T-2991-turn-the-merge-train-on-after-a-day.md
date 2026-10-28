---
id: T-2991
title: "Turn the merge train on after a day of shadow runs by the merger routine"
milestone: M2
lane: harness
status: blocked
priority: P1
size: S
tier: opus
depends_on: [T-2989]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
After at least a day of shadow runs by the owner's merger routine, all agreeing with what the PRs
did, the switch moves to `mode = "train"` and the merge train is live.

## Context
- SPEC-0027 Rollout, "Shadow evidence for T-2991": the evidence comes from the GitHub API, not
  from branch history, because the branch is force-pushed:
  - `gh run list --workflow ci.yml --branch merge-train/shadow --created ">=<date>" --json
    headSha,conclusion,createdAt` lists the runs;
  - the commits API reads each `headSha` for the batch and its predicted outcome;
  - `make merge-train-stats` (T-2995) prints the same summary.
- The bar:
  - at least 10 shadow trains over at least 24 h, the newest within the last 2 h;
  - every prediction agrees with what the PRs did under the per-PR flow;
  - no tree difference predicted;
  - the red-train rate is reported against p = 0.09.

## Blocked because
This needs the owner's merger routine (SPEC-0027 "Owner steps"), in addition to the dependency on
T-2989. To check whether the blocker is gone (decision 0013), run
`gh run list --workflow ci.yml --branch merge-train/shadow --limit 50 --json createdAt`. It must
show runs spanning at least 24 h, with one in the last 2 h. Until then the routine is missing or
shadow mode has not run long enough.

## Acceptance criteria
- [ ] The PR states the shadow evidence: train count, agreement, run links.
- [ ] `scripts/merge_train.toml` says `mode = "train"`. A short decision record cites the evidence
      and T-2996's record.
- [ ] At Outcome time, the Outcome links any live trains that ran during this session. The first
      live trains are checked by T-2990, so this task does not wait for them. Any tree-check
      failure means the emergency switch back to `lock`, with a P0 task.

## Out of scope
- The metric review (T-2990).
