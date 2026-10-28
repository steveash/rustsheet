---
id: T-2985
title: "Make the merge lock fair: a FIFO waiter list and a wait command"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2984]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge lock serves waiters first-in, first-out. A waiter registers in the lock commit, a free,
stale or finished lock goes to the head of the queue, and a holder that releases rejoins at the
tail. No session can then chain merges past waiters. A `wait` command replaces polling.

## Context
- SPEC-0027 section 1 ("A fair merge lock") is the design:
  - the commit-message fields;
  - the `acquire` steps;
  - `WAITER_TTL` = 10 min, with refresh writes at most every 3 min;
  - `wait`, polling every 30 s with a timeout;
  - train locks (`pr: train`, `prs: ...`), which count as finished only when all their PRs are.
- Evidence: over the 48 h before 2026-10-25, #1860 and #2000 waited 30–40 min while one session
  chained #1859 and #1865. The lock was taken over 37 times (SPEC-0027 Problem).
- Code to change: `scripts/merge_lock.py`, `scripts/test_merge_lock.py`, the Makefile's
  merge-lock targets, the `steward` skill's "Merge lock" paragraph, and work-loop step 6.3.
- This slice is independent of the rest of SPEC-0027 and helps in every mode.

## Acceptance criteria
- [ ] `merge_lock.py` keeps the holder and the queue in one lock commit. It implements SPEC-0027
      section 1's `acquire`, `release`, `wait` and `--train/--prs`, and still parses old-format
      lock commits.
- [ ] `make merge-lock-wait PR=<n> [TIMEOUT=45]` exists, and `make merge-lock-status` prints the
      queue.
- [ ] `test_merge_lock.py` covers every `test_merge_lock.py` case in SPEC-0027's Test plan.
- [ ] Step 1 of the steward skill's "Merge lock" and work-loop step 6.3 say to wait with `make
      merge-lock-wait` in the background instead of retrying later.
- [ ] A decision record, "The merge lock serves waiters first-in, first-out", lands in this PR.
      It amends 0075 and 0097, with the text from SPEC-0027 "Decision records and governance".
- [ ] `make hygiene`, `make typos` and `check_governance.py origin/main` pass. The architecture
      and security reviewers explicitly approve the protected paths.

## Out of scope
- The readiness marker and the merge train (T-2986, T-2987).
- Changing the 30-minute TTL.
