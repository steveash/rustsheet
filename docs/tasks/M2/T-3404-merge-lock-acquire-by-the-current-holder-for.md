---
id: T-3404
title: "Merge lock: acquire by the current holder for a different PR must queue, not silently retarget the held lock"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
One session can no longer chain a second merge past the FIFO queue: a session that holds the merge
lock for PR A and runs `acquire` or `wait` for PR B joins the tail of the queue for B (or is refused)
instead of having its held lock renewed and relabelled to B.

## Context
- Seen on 2026-10-31: a session held the lock for #2539 while a background `merge_lock.py wait --pr
  2550` from the same session (same `CLAUDE_CODE_SESSION_ID` holder) ran. The wait printed "merge lock
  renewed for PR 2550", and the lock's `pr` field switched to 2550 while #2539 was still being merged,
  ahead of four waiters (the session then released and re-queued #2550 by hand).
- Cause: `attempt()` in `scripts/merge_lock.py` treats "the lock's holder is me" as a renewal whatever
  `pr` it was called with. That breaks decision 0145's "a holder that releases rejoins the tail" and
  SPEC-0027 section 1's fairness goal whenever one session works several PRs.
- Exit 4 (`--renew`) is unaffected: a renewal states the PR it holds.

## Acceptance criteria
- [ ] `acquire --pr B` or `wait --pr B` by the holder of a lock held for PR A (A != B) leaves the held
      lock unchanged (holder, `pr`, `renewed`), registers a waiter for B at the tail of the queue, and
      returns BUSY (exit 1) with a message naming both PRs (behaviour chosen in advance, so no design
      call is left). Note it in decision 0145's status line.
- [ ] `acquire --pr A` (with or without `--renew`) by the holder of A still renews and keeps the queue
      order, as today.
- [ ] `scripts/test_merge_lock.py` covers both, plus a train holder (`--train ID`) asking for a PR.
- [ ] `make hygiene` passes; reviewers approve (architecture and security for the protected path).

## Out of scope
- The merge train (T-2987 onward).
