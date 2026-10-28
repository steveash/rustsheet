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
The merge lock serves waiters first-in, first-out. A waiter registers in the lock commit, and a
free, stale or finished lock goes to the head of the queue. A holder that releases rejoins at the
tail, so no session can chain merges past waiters. A `wait` command replaces polling.

## Context
- SPEC-0027 section 1 ("A fair merge lock") is the design:
  - the version-2 commit format, validated on write and on parse:
    - id and number regexes, no newlines, at most 20 waiters, `prs` capped;
    - timestamps more than 60 s past the lock commit's committer time are clamped to that time,
      never to the read time;
    - out-of-range waiter timestamps expire the waiter;
    - a missing or malformed holder reads as held (stale after the TTL), never free;
  - the `acquire` steps;
  - `WAITER_TTL` = 10 min, with refreshes written at most every 3 min;
  - a head that does not claim a free lock within 2 min of `freed` is dropped;
  - a rate-limited PR-finished read (head changes or every 10th poll);
  - train locks (`pr: train`, `prs`): finished only when `prs` is non-empty and all of them are
    finished. An empty `prs` is never finished, only stale after the TTL;
  - renew and release retry until they succeed or the lock is no longer theirs; a lost holder
    gets exit code 4;
  - `wait` every 30 s, with a timeout;
  - mixed-version clients during the rollout (fairness loss only, no loss of mutual exclusion)
    and clock skew.
- Evidence: over the 48 h before 2026-10-25, #1860 and #2000 waited 30–40 min while one session
  chained #1859 and #1865. The lock was taken over 37 times (SPEC-0027 Problem).
- Code to change: `scripts/merge_lock.py`, `scripts/test_merge_lock.py`, the Makefile's
  merge-lock targets, the `steward` skill's "Merge lock" paragraph, and work-loop step 6.3.
- This slice keeps `merge_lock.py`'s urllib PR-finished read, now rate-limited. T-2993 later moves
  that read onto `scripts/gh_client.py`.
- If the diff grows past size M, split it before review: the format and validation first, then
  `wait` and the skill text.

## Acceptance criteria
- [ ] `merge_lock.py` implements SPEC-0027 section 1: the version-2 format and its validation, the
      FIFO queue, `acquire --pr/--train --prs`, exit code 4, `release` and `wait`. It still parses
      version-1 commits.
- [ ] `make merge-lock-wait PR=<n> [TIMEOUT=45]` exists, and `make merge-lock-status` prints the
      queue.
- [ ] `test_merge_lock.py` covers every `test_merge_lock.py` case in SPEC-0027's Test plan,
      including these: an empty-`prs` train lock is never finished; a renewal that loses its CAS
      to a waiter's refresh retries and succeeds, and exit 4 follows a takeover; newline
      injection; future timestamps clamped to the committer time; a malformed holder reads as
      held.
- [ ] Step 1 of the steward skill's "Merge lock" and work-loop step 6.3 say to wait with
      `make merge-lock-wait` in the background, instead of retrying later.
- [ ] A decision record, "The merge lock serves waiters first-in, first-out", lands in this PR.
      It amends 0075 and 0097 and cites SPEC-0027 section 1.
- [ ] `make hygiene`, `make typos` and `check_governance.py origin/main` pass. The architecture
      and security reviewers approve the protected paths explicitly.

## Out of scope
- The readiness marker and the merge train (T-2986, T-2987).
- Changing the 30-minute TTL.
