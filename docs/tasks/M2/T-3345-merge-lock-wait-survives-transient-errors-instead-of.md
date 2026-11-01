---
id: T-3345
title: "merge_lock wait survives transient errors instead of exiting 2"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`scripts/merge_lock.py wait` keeps waiting through transient failures. A waiter at the head of
the queue that dies leaves the lock reserved for nobody until its reservation goes stale, which
stalls every agent behind it.

## Context
On 2026-10-30, while eight stranded PRs were driven through the lock queue (15 waiters at peak),
`merge_lock.py wait --pr N --timeout 240` exited with code 2 three times, for #2371 (twice) and
#2344. Each time the waiter was still queued, and #2344 was at the head of the queue. stderr was
not captured, so the exact error is not known.

`wait()` handles errors like this:
- it retries a `GitError`, but only up to `GIT_FAILURES = 3` in a row;
- `main()` maps any `GitError` or `LockFormatError` to exit 2 (`ERROR`);
- a `LockFormatError` raised inside `attempt()` is not retried at all.

The likely causes, under a busy queue, are:
- three straight CAS losses or push failures, or proxy hiccups, on the `merge-lock` branch;
- a format error read while another writer updates the marker.

T-3010 built `wait`.

## Acceptance criteria
- [ ] Reproduce or pin down the failure: a test that injects 3+ consecutive `GitError`s, and a
      `LockFormatError` from a racing write, into `wait`.
- [ ] Errors are classified explicitly, for example with a distinct exception type for permanent
      errors, so a persistent `LockFormatError` is never retried forever:
      - **Permanent:** a bad holder id, a bad PR number, or a full queue (`QUEUE_MAX` waiters)
        when it first joins. These exit 2 at once.
      - **Transient:** git and push failures, CAS loss, and a marker unreadable during a racing
        write. `wait` retries these with backoff until its timeout and prints each distinct
        error.
      - **Repeated unreadable marker:** an unreadable marker that persists across, say, 5
        consecutive polls is reported and exits 2.

      A queue that becomes full while the waiter is already queued does not end its wait.
- [ ] The waiter keeps its queue place across retries: a test asserts its id is still in the
      marker's queue, at the same position, after an injected error.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lock throughput and the merge train (SPEC-0027).

## Outcome
`merge_lock.py wait` now checks the claim up front (bad id/PR exits 2), exits 2 at once when the
queue is full on first join, retries `GitError`s until its timeout, and retries an unreadable marker,
raising only after `UNREADABLE_POLLS` (5) in a row. `GIT_FAILURES` is gone. Tests cover each case and
that the waiter keeps its queue place. No eval delta (harness script). Decision 0189 records the rule.

## Handoff (resumed)
Resolved by the opus resume: decision 0189 records the protected-path change, the panel re-ran on
the new head, and the structured-status follow-up is T-3376.
