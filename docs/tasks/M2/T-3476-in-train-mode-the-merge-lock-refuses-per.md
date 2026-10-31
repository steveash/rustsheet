---
id: T-3476
title: "In train mode the merge lock refuses per-PR claims except the emergency-flip PR"
milestone: M2
lane: harness
status: done
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
- [x] In `train` mode (`merge_train.read_mode`, the same read as `make merge-mode`), `acquire
      --pr N` and `wait --pr N` exit 5 with a message naming `make mark-ready`, unless PR N
      carries `merge-train-stop`.
      - `--train` claims are unaffected.
      - `release` and `status` are unaffected.
      - In `lock` and `shadow` mode nothing changes.
- [x] A failed read of the mode (which reads as `lock`) or of the PR's labels lets the claim
      proceed. An emergency flip must never be blocked by a GitHub read.
- [x] Tests cover:
      - refusal in `train` mode;
      - the flip-PR exception;
      - `lock` and `shadow` modes;
      - a failed label read;
      - `--train` claims.
- [x] The skills and the Makefile help name exit 5. A decision record covers the protected-path
      change.
- [x] `make check` green; reviewers approve.

## Out of scope
- Creating the merger routine (an owner step).
- Queue priority for trains.

## Outcome
- **The guard:** `scripts/merge_lock.py` `train_mode_refusal()`. `acquire --pr` and `wait --pr`
  exit 5 (`REFUSED`) in `train` mode unless the PR carries `merge-train-stop`. The mode is
  `merge_train.read_mode` (`make merge-mode`'s read). An unreadable mode or label lets the claim
  through. The claim is validated first, so a malformed holder is still exit 2.
- **Tests:** a new test in `test_merge_lock.py` covers:
  - refusal of `acquire` and `wait`, leaving the lock untouched;
  - the flip-PR exception;
  - an unreadable label;
  - `lock` and `shadow` modes;
  - a `--train` claim.

  All 40 lock tests pass.
- **Live check** on 2026-10-31 (in `train` mode): `merge_lock.py acquire --pr 2593` exits 5 with
  the message. At the same time another session was still queued per-PR for #2598.
- **Docs:** decision 0204; the `work-loop` `train` bullet and the `steward` "Merge lock" step name
  exit 5; the Makefile help for `merge-lock-acquire` and `merge-lock-wait` too.

