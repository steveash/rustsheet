---
id: T-2985
title: "Make the merge lock fair: a FIFO waiter list and a wait command"
milestone: M2
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2984]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge lock serves waiters first-in, first-out. A waiter registers in the lock commit, and a
free, stale or finished lock goes to the head of the queue. A holder that releases rejoins at the
tail, so no session can chain merges past waiters. (`wait` and the skill text are T-3010.)

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
  - train locks (`pr: train`, `prs`): finished only when `prs` is non-empty and all of them are
    finished. An empty `prs` is never finished, only stale after the TTL;
  - renew and release retry until they succeed or the lock is no longer theirs; a lost holder
    gets exit code 4;
  - mixed-version clients during the rollout (fairness loss only, no loss of mutual exclusion)
    and clock skew.
- Evidence: over the 48 h before 2026-10-25, #1860 and #2000 waited 30–40 min while one session
  chained #1859 and #1865. The lock was taken over 37 times (SPEC-0027 Problem).
- Code to change: `scripts/merge_lock.py`, `scripts/test_merge_lock.py`, and the Makefile's
  `merge-lock-status` target.
- This slice keeps `merge_lock.py`'s urllib PR-finished read. T-3001 later moves that read onto
  `scripts/gh_client.py`.
- Split at the seam named here when the first draft passed size M: this task keeps the format,
  validation, queue, `acquire`, renew/release retry, exit 4 and `status`; T-3010 takes `wait`,
  the rate-limited PR-finished read, `make merge-lock-wait` and the skill text.
- `acquire --renew` (not in the spec) lets a caller state it holds the lock, so a takeover
  between two calls exits 4 instead of reading as busy; the merger (T-2994) needs it.

## Acceptance criteria
- [x] `merge_lock.py` implements SPEC-0027 section 1: the version-2 format and its validation, the
      FIFO queue, `acquire --pr/--train ID --prs`, exit code 4, renew/release retry and
      `release`. It still parses version-1 commits.
- [x] `make merge-lock-status` prints the queue.
- [x] `test_merge_lock.py` covers every `test_merge_lock.py` case in SPEC-0027's Test plan (the `wait` and
      rate-limit cases are T-3010's),
      including these: an empty-`prs` train lock is never finished; a renewal that loses its CAS
      to a waiter's refresh retries and succeeds, and exit 4 follows a takeover; newline
      injection; future timestamps clamped to the committer time; a malformed holder reads as
      held.
- [x] A decision record, "The merge lock serves waiters first-in, first-out", lands in this PR.
      It amends 0075 and 0097 and cites SPEC-0027 section 1.
- [x] `make hygiene`, `make typos` and `check_governance.py origin/main` pass. The architecture
      and security reviewers approve the protected paths explicitly.

## Out of scope
- `wait`, the rate-limited PR-finished read, `make merge-lock-wait` and the skill text (T-3010).
- The readiness marker and the merge train (T-2986, T-2987).
- Changing the 30-minute TTL.

## Outcome
- `scripts/merge_lock.py` writes and reads the version-2 lock commit: holder, PR or train `prs`,
  `renewed`, `freed` and up to 20 waiters, validated on write and on parse (fails closed), with
  timestamps clamped to the committer time. Version-1 commits still parse. `acquire` queues a busy
  caller at the tail and hands a free, stale or finished lock only to the head; renew and release
  retry their CAS; a lost holder exits 4 (`--renew` states a caller holds it). Each PR's state is
  read at most once per `acquire`. `make merge-lock-status` prints the queue.
- 32 tests in `scripts/test_merge_lock.py` (15 new), isolated from the real origin by a git guard.
- Decision 0145 amends 0075 and 0097. Split along the task's own seam: `wait`, the cross-poll
  rate limit and the skill text are T-3010.
- During development one test run pushed one waiter line to the real `merge-lock` branch (the
  holder was kept; the next holder's write replaced it). The guard now makes that impossible.
- No eval delta (no engine change).
