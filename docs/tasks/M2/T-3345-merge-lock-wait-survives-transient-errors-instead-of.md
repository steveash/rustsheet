---
id: T-3345
title: "merge_lock wait survives transient errors instead of exiting 2"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
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
- [ ] `wait` retries transient failures (git, CAS loss, unreadable marker) with backoff until
      its timeout, and prints each distinct error. It exits 2 only for errors that cannot be
      transient: a bad holder id, a bad PR number, or a queue over `QUEUE_MAX` that persists.
- [ ] The waiter keeps its queue place across retries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lock throughput and the merge train (SPEC-0027).
