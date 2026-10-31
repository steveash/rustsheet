---
id: T-3476
title: "In train mode the merge lock refuses per-PR claims except the emergency-flip PR"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
In `train` mode, `merge_lock.py acquire --pr N` and `wait --pr N` refuse, with a clear message, any
per-PR claim except the emergency-flip PR's. A session still on the per-PR flow then marks its PR
ready instead of queueing ahead of the trains.

## Context
On 2026-10-31 the repo went to `train` mode at 10:20 UTC (T-2991, #2547). In the 7 hours after,
the lock went to:
- about 12 train holds;
- 27 per-PR holds: 23 from other sessions and 4 from the session that filed this task.

They came from long-running interactive sessions and worker runs that started before the switch,
and kept the per-PR flow. Each per-PR hold lasts a full CI run, so trains waited behind them, and
13 ready PRs were stranded for more than 2 h, which is the `train`-mode threshold for the emergency
flip.

The skill already says agents in `train` mode "do not take the lock". Its one per-PR exception is
the emergency-flip PR (`merge-train-stop`, `steward` skill "Emergency flip to `lock`" step 4).
Nothing enforces this.

## Acceptance criteria
- [ ] In `train` mode (`merge_train.read_mode`, the same read as `make merge-mode`), `acquire
      --pr N` and `wait --pr N` exit 5 with a message naming `make mark-ready`, unless PR N
      carries `merge-train-stop`.
      - `--train` claims are unaffected.
      - `release` and `status` are unaffected.
      - In `lock` and `shadow` mode nothing changes.
- [ ] A failed read of the mode (which reads as `lock`) or of the PR's labels lets the claim
      proceed. An emergency flip must never be blocked by a GitHub read.
- [ ] Tests cover:
      - refusal in `train` mode;
      - the flip-PR exception;
      - `lock` and `shadow` modes;
      - a failed label read;
      - `--train` claims.
- [ ] The skills and the Makefile help name exit 5. A decision record covers the protected-path
      change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Creating the merger routine (an owner step).
- Queue priority for trains.
