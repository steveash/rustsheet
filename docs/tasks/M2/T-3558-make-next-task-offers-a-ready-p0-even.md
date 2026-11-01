---
id: T-3558
title: "make next-task offers a ready P0 even when its lane is busy, and says 'lane busy' rather than 'no ready task' when only the busy-lane rule hides ready tasks"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A ready P0 task is offered by `make next-task` / `make ready-tasks` even when its lane is busy, so a P0 never waits
for unrelated work in its lane to merge. When a lane's ready tasks are hidden only by the one-task-in-flight rule,
`next` prints that the lane is busy and names the branch in flight, instead of `no ready task`. Workers then stop
launching the planner for lanes that have work queued.

## Context
- Observed 2026-11-01 on origin/main b39e1eae. T-3488 (docs/tasks/M2/T-3488-*.md) is `P0`, `todo`, `tier: opus`,
  with no dependencies. It is the merge train's stranded-PR fix that the steward filed when it flipped the train to
  `lock` (docs/status.md, 18:43 UTC snapshot). Yet `make next-task LANE=harness TIER=opus` prints `no ready task`:
  `branch_claims` in scripts/tasks.py finds the harness lane active with `agent/harness/T-3358-gate-a-tables` and
  `agent/harness/T-3543-gate-a-discrete`, so `busy` drops every harness candidate, P0s included.
- The same run showed `no ready task` for engine (T-3211 in flight) and research (T-3298 in flight). The M2 queues
  still held 11 engine, 8 harness and 24 research tasks that ignore claims (status.md "Queues"). The work-loop skill,
  step 1, then tells the worker to launch the planner, so each busy lane triggers a planner run whose new tasks are
  hidden by the same rule.
- Rules in play:
  - Decision 0022: one task in flight per lane, enforced by `make next-task`.
  - Decision 0023: "P0 tasks (any milestone), then unresumed handoffs, ..."; its Consequences say a P0 "is never
    delayed by an escalation".
  - Decision 0018: busy means a branch commit within the claim window, handoffs excepted.
  - The ordering and busy filter are in `available` in scripts/tasks.py; `main` prints `no ready task` for `next`. Tests
    are in scripts/test_tasks.py.
- Design question for this task: whether a P0 bypasses the busy rule outright, or only when no P0 of that lane is
  already in flight. T-3358, in flight here, is itself P0, so the second rule would still hide T-3488. Decide it, with
  the conflict risk the busy rule guards against, in a decision record. scripts/ and `.claude/` are protected paths:
  the PR needs that decision record either way.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) states the P0 rule and the new `next` message, and amends decisions
      0022/0023 by reference.
- [ ] `python3 scripts/tasks.py next --lane harness --tier opus` offers a ready P0 whose lane is busy, under the
      decided rule. A test in scripts/test_tasks.py covers it with a busy lane, a P0 and a non-P0 candidate.
- [ ] When the lane has ready candidates of the tier and all of them are hidden by the busy rule, `next` prints
      `lane <lane> busy: <task id> in flight on <branch>` and exits with a status distinct from the empty-queue
      one. A test covers it. `ready` output is unchanged, apart from the P0 rule.
- [ ] The work-loop skill's step 1 (`.claude/skills/work-loop/SKILL.md`) says not to launch the planner or the
      gap-triager on a `lane ... busy` result. Same PR, under the same decision record.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3488's merge-train fix itself.
- Changing the claim window or how handoffs and review-limited branches free a lane (decisions 0018, 0095).
- The duplicate id T-3543, used on both `agent/harness/T-3543-gate-a-discrete` and `claude/id-registry`. The
  id-allocation tasks own it (T-3543, the atomic id registry, done).
