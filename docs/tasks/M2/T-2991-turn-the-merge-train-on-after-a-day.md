---
id: T-2991
title: "Turn the merge train on after a day of shadow runs by the merger routine"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-2989]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge train goes live. The owner's merger routine first runs in shadow mode for at least a
day, and its trains must agree with what the PRs then did. Then the switch is set to
`mode = "train"`, and the first live trains are watched.

## Context
- SPEC-0027 Rollout step 7 and the Test plan's "Live rollout checks": at least 10 shadow trains
  whose predicted outcome agrees with the per-PR flow, and no tree-check difference.
- This needs the owner's merger routine (SPEC-0027 "Owner steps"). To prove it runs, run
  `git fetch origin merge-train/current && git log -3 --format='%ci %s' origin/merge-train/current`.
  It must show shadow trains pushed within the last 2 hours, and shadow trains going back at least
  24 hours. If it does not, block this task and give that command as the check (decision 0013).

## Acceptance criteria
- [ ] The PR gives the shadow evidence: how many trains ran, how many agreed, and the run links.
- [ ] `scripts/merge_train.toml` says `mode = "train"`. A short decision record cites the
      evidence and T-2989's record.
- [ ] The first two live trains merged their batches and every tree check passed, with links in
      the task's Outcome. Or the mode was set back to `lock`, with a P0 task filed.

## Out of scope
- The metrics review (T-2990).
