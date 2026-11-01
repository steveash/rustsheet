---
id: T-3376
title: "merge_lock attempt returns a structured queue status instead of wait matching message text"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`merge_lock.py wait` learns whether its caller is queued, or could not join a full queue, from a
value `attempt` returns, not from the wording of the message `busy_message` builds.

## Context
T-3345 (decision 0189) made `wait` exit 2 when the queue is full on first join and keep waiting
once queued. It tells the two apart by matching `busy_message` text: `message.endswith(QUEUE_FULL)`
and `IN_QUEUE in message` (`scripts/merge_lock.py`, `wait`). The shared constants keep both sides
in sync today, but rewording the message would silently change control flow. All three reviewers
of #2513 named this as a follow-up.

## Acceptance criteria
- [ ] `attempt` (or a small result type it returns) carries the caller's queue place, or that the
      queue was full, alongside the exit code and message; `wait` branches on that value only.
- [ ] `QUEUE_FULL` / `IN_QUEUE` are no longer used for control flow (keep them only if a message
      still needs them).
- [ ] The existing `scripts/test_merge_lock.py` wait tests pass unchanged in what they assert
      (exit codes, printed lines, queue place); `acquire`'s CLI output is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to the lock format, the queue rules or the retry policy (decisions 0145, 0189).
