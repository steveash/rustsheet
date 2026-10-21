---
id: T-2285
title: "The merge lock is never held for a PR that is already merged or closed"
milestone: M5
lane: harness
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A session that keeps renewing the merge lock after its PR merged no longer blocks every other merge
in the repository.

## Context
- On 2026-10-21 holder `16664e56-…` kept renewing the lock for PR 1410 from 06:33 UTC, when #1410
  merged, until at least 06:52. Each renewal reset the 30-minute TTL (decision 0075), so nobody could
  take it over, and at least two other sessions were waiting ("merge lock held by #1410, retry 06:59").
  The owner's session took it over with `scripts/merge_lock.py acquire --ttl 0`.
- scripts/merge_lock.py `acquire` renews a lock its holder already holds without checking the PR.

## Acceptance criteria
- [ ] `acquire --pr N` refuses to take or renew the lock when PR N is merged or closed (one GitHub API
      read; if the read fails, behave as today), and prints why.
- [ ] Another acquirer may take over a lock whose PR is merged or closed, whatever its age, and says so.
- [ ] The steward skill's merge-lock paragraph mentions both rules (protected path: decision record).
- [ ] Unit tests in scripts/test_merge_lock.py; `make check` green; reviewers approve.

## Out of scope
- Changing the TTL.
