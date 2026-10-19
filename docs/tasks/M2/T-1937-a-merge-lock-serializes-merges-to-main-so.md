---
id: T-1937
title: "A merge lock serializes merges to main, so a green PR is not overtaken while its CI runs"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Agents merge to `main` one at a time: a merger holds a lock while it brings `main` into its PR, waits for CI on that head and merges, so `main` cannot move under a green PR. This replaces the merge queue (decision 0053), which a repository owned by a personal account cannot turn on.

## Context
Owner request (2026-10-19). Decision 0028 requires the merged head to contain the current `main`; with about 50 merges a day and a
5-minute CI, `main` moves during a PR's CI run: T-1861's PR #1115 needed five merges of `main` before one landed. The merge queue
cannot be turned on for `steveash/rustsheet` (a personal account), so the agents serialize themselves.
Design: the lock is the branch `merge-lock` on origin, an orphan commit with an empty tree whose message names the holder (session,
PR, time). `git push --force-with-lease=refs/heads/merge-lock:` creates it only if it is absent, atomically on the server; release
deletes it only if it still holds the holder's commit. A lock whose commit is older than the TTL may be taken over (also
compare-and-swap). The empty tree means no task or decision file is seen on it (tasks.py scans every origin branch) and CI does not run
(ci.yml runs on pull requests and on `main`). Code: new scripts/merge_lock.py with tests; Makefile targets; the steward skill's merge
policy and the work-loop step 6.3; a decision record amending 0028/0053.

## Acceptance criteria
- [ ] `make merge-lock-acquire PR=<n>` takes the lock only when no live lock exists (two racing acquirers: exactly one wins), `make merge-lock-release` drops only the caller's own lock, a lock older than the TTL is taken over, and `make merge-lock-status` shows the holder; unit tests run each case against a real temporary remote.
- [ ] The steward skill's merge policy and the work-loop say: acquire, merge `main` in, push, wait for CI on that head, merge, release; release at once on a red CI or any stop; never merge without the lock; what to do while another agent holds it.
- [ ] A decision record amends 0028 and 0053.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
