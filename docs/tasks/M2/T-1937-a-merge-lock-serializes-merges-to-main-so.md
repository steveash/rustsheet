---
id: T-1937
title: "A merge lock serializes merges to main, so a green PR is not overtaken while its CI runs"
milestone: M2
lane: harness
status: done
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
- [x] `make merge-lock-acquire PR=<n>` takes the lock only when no live lock exists (two racing acquirers: exactly one wins), `make merge-lock-release` drops only the caller's own lock, a lock older than the TTL is taken over, and `make merge-lock-status` shows the holder; unit tests run each case against a real temporary remote.
- [x] The steward skill's merge policy and the work-loop say: acquire, merge `main` in, push, wait for CI on that head, merge, release; release at once on a red CI or any stop; never merge without the lock; what to do while another agent holds it.
- [x] A decision record amends 0028 and 0053.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`scripts/merge_lock.py` (`make merge-lock-acquire PR=<n>`, `merge-lock-release`, `merge-lock-status`) keeps the lock as the branch
`merge-lock` on origin, an orphan empty-tree commit naming the holder (`CLAUDE_CODE_SESSION_ID`) and the PR. Every write is a
`--force-with-lease` compare-and-swap, so of two racing acquirers one wins, only the holder releases, and a lock older than 30 minutes
is taken over. 10 unit tests run against a real temporary bare remote. Decision 0075 sets the rule; the steward skill ("Merge lock"),
the work-loop step 6.3 and agent-workflow.md step 9 follow it, 0028 and 0053 point to it, and the queue instructions are marked
dormant. Review: security, architecture and quality APPROVE; their minor findings are fixed (one-fetch read of the lock, renew before
the merge call, a numeric `--pr`, the cross-references), except a shared git helper for scripts/ (the wrappers differ; the duplication
check shows no hit). Eval: no change (no engine code).
