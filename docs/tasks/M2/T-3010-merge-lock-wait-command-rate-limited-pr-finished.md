---
id: T-3010
title: "Merge lock wait command, rate-limited PR-finished read, and skills that wait instead of retrying"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-2985]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A session that finds the merge lock busy waits for its turn with one background command instead
of retrying later, and waiting costs few GitHub API reads.

## Context
- Second slice of SPEC-0027 section 1, split from T-2985 when that diff passed size M. T-2985
  lands the version-2 format, the FIFO queue, `acquire`, renew/release retry and exit 4;
  decision 0145 already describes `wait`.
- `wait`: poll every 30 s, print only changed messages, default timeout 45 min; exit 0 held,
  3 PR done, 1 timeout. Each poll refreshes the waiter's `seen` at most every 3 min (T-2985's
  rule), so a waiter stays queued.
- The PR-finished read (urllib, kept until T-3001 moves it onto `scripts/gh_client.py`) is
  re-read only when the head changes or every 10th poll, cached per PR (the caller's own, the
  holder's and the head's), about 12 reads an hour per PR. Hoisting the reads (at most one per PR
  per `acquire`, outside the CAS retries) is done in T-2985, so T-3010 adds the per-PR cross-poll
  cache.
- A draft of both is in T-2985's working notes: the `wait` loop and a `FinishedReads` cache.

- Left from T-2985's review: SPEC-0027 section 1 step 1 should say the finished head is
  dropped only when the lock is free, stale or finished (as `attempt()` does), and
  `make merge-lock-acquire` should pass `--renew` through (`RENEW=1`) or stop saying "renew".

## Acceptance criteria
- [x] `scripts/merge_lock.py wait --pr N [--timeout MIN]` implements SPEC-0027 section 1's
      `wait`, and `make merge-lock-wait PR=<n> [TIMEOUT=45]` runs it.
- [x] The PR-finished read is rate-limited per PR as above.
- [x] `scripts/test_merge_lock.py` covers `wait` reaching 0, 3 and the timeout, and the rate
      limit (reads counted with a fake clock and API).
- [x] Step 1 of the steward skill's "Merge lock" and work-loop step 6.3 say to wait with
      `make merge-lock-wait` in the background, instead of retrying later.
- [x] SPEC-0027 section 1 step 1 and the `merge-lock-acquire` target match the code (above).
- [x] `make hygiene`, `make typos` and `check_governance.py origin/main` pass; the architecture
      and security reviewers approve the protected paths explicitly.

## Out of scope
- The format, queue and acquire logic (T-2985).
- Moving the PR-finished read onto `gh_client.py` (T-3001).

## Outcome
- `scripts/merge_lock.py wait --pr N | --train ID [--prs ...] [--timeout 45]` polls `acquire`
  every 30 s, prints only changed messages and exits 0 (held), 3 (PR done), 1 (timeout) or 2
  (three git errors in a row). `--timeout` and `--ttl` refuse NaN, inf and negatives.
- `FinishedReads` caches PR-finished reads per PR on top of T-2985's per-`acquire` memo: across
  polls a PR is re-read only every 10th poll, so a new head or holder is read at once.
- `make merge-lock-wait PR=<n> [TIMEOUT=45]`; `make merge-lock-acquire ... RENEW=1` passes
  `--renew`. The steward skill ("Merge lock" steps 1 and 3), work-loop step 6.3 and
  agent-workflow step 9 wait with `make merge-lock-wait` in the background; decisions 0097 and
  0145 describe the current state.
- Picked up from T-2985's review: SPEC-0027 section 1 step 1 now says the finished head is dropped
  only when the lock is free, stale or finished; `RENEW=1` on `merge-lock-acquire`.
- `scripts/test_merge_lock.py`: 37 tests (wait reaching 0, 3 and the timeout, git-error retry, the
  rate limit counted with a fake clock and API, `main wait` for a PR and a train, bad minutes).
- No eval delta (no engine change).
