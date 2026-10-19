---
id: T-1938
title: "The merge lock releases by writing a free marker, since the session git proxy refuses branch deletion"
milestone: M2
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`make merge-lock-release` frees the merge lock from an agent session: today it reports "taken over before release" and the lock stays held until its 30-minute TTL.

## Context
Found on the lock's first live use (T-1937, PR #1128): the release push deletes `merge-lock`, and the session git proxy answers a
branch deletion with HTTP 403 (a policy denial), while it allows creating the branch and a `--force-with-lease` update of it. The
local bare-remote tests could not see this. Fix: never delete; release writes a "free" marker commit with the same compare-and-swap,
and a free marker reads as no lock. Code: scripts/merge_lock.py, scripts/test_merge_lock.py, decision 0075.

## Acceptance criteria
- [x] Release leaves the branch in place with a free marker; status reads free; acquire takes a free lock by compare-and-swap; no code path deletes the branch.
- [x] Verified live against origin from an agent session: acquire, status, release, status.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Release writes a free marker commit (`state: free`) with the same `--force-with-lease` compare-and-swap; `read_lock` reports it as
free, and acquire swaps it out. No push deletes the branch (a test spies on every push). 12 unit tests. Live from an agent session
against origin on 2026-10-19: status (held by this session for PR 1128) -> release -> free -> acquire PR 0 -> held -> release -> free;
the branch stays with the marker. Decision 0075 updated.
