---
id: T-2991
title: "Turn the merge train on after a day of shadow runs by the merger routine"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-2989, T-3055]
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

## Acceptance criteria
- [x] The PR gives `make merge-train-stats`'s shadow evidence against every point of the bar,
      with run links.
- [x] `scripts/merge_train.toml` says `mode = "train"`. A short decision record cites the evidence
      and T-2996's record.
- [x] The Outcome links any live trains that ran during this session. The first live trains are
      checked by T-2990 at its checkpoint, so this task does not wait for them. A tree-check
      failure at any time means the emergency flip back to `lock`, with a P0 task.

## Out of scope
- The metric review (T-2990).

## Outcome
2026-10-31: the merge train is live (`mode = "train"`, decision 0192).

Evidence from `make merge-train-stats` at 08:07 UTC (T-3397's parallel reads):
- 42 scored shadow trains over 22 h (the owner waived the remaining 2 h of the 24 h span);
- newest shadow run https://github.com/steveash/rustsheet/actions/runs/41181616335; 50 of 51
  shadow CI runs green, 1 cancelled;
- 0 tree-check failures; 0 red trains in 47;
- PR CI p90 7.8 min against 6.5 min before; queue p90 3 s;
- agreement 87% (151/174).

The owner approved the switch below the 90% agreement point and before the 24 h mark. The 23 disagreements are explained
in decision 0192: 12 stranded, 6 changed after marking, 5 batch order.

Live trains: none ran during this session. T-2990 checks the first ones at its checkpoint.
