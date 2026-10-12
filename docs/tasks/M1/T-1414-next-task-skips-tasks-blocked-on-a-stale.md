---
id: T-1414
title: "next-task skips tasks blocked on a stale pushed branch; M14 desktop tasks stay blocked"
milestone: M1
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`make next-task` never returns a task that cannot progress: one that is `status: blocked` on its pushed
agent branch (at any branch age), or an M14 desktop-oracle task while `excel-desktop` is n/a.

## Context
On 2026-10-12 `make next-task LANE=functions TIER=opus` returned T-1186, and the opus ready list held
T-1155: both are `status: blocked` on their pushed branches (#436, #409), with reasons recorded, but the
branches are older than `CLAIM_TTL_SECONDS` (24 h), so `branch_claims` no longer counts them as claims
and main still says `todo`. A worker following the queue would restart a task whose blocker
(e.g. T-1187, closed as "not found") still holds. `make next-task LANE=research TIER=opus` returned
T-1204, an M14 task that decision 0039 says stays `status: blocked` until `excel-desktop OK`; T-1277 and
T-1299 are in the same state. Code: `scripts/tasks.py` `branch_claims`, which already treats handoffs
as claims at any age.

## Acceptance criteria
- [ ] A task whose file on a pushed `agent/*` branch says `status: blocked` is claimed at any branch age;
  `scripts/test_tasks.py` covers a stale blocked branch.
- [ ] T-1204, T-1277 and T-1299 are `status: blocked` on main, citing decision 0039.
- [ ] Decision record for the protected `scripts/` change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unblocking or re-scoping T-1155/T-1186 themselves.
