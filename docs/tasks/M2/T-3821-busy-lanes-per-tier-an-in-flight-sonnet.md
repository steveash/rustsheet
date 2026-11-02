---
id: T-3821
title: "Busy lanes per tier: an in-flight sonnet branch no longer hides the lane's ready opus tasks, and the reverse"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3558]
---

## Goal
A lane is busy for one tier only. A branch in flight on a `tier: sonnet` task stops `make next-task` from offering
another sonnet task in its lane, but it no longer hides that lane's ready opus tasks, and an opus branch in flight
no longer hides the lane's sonnet tasks. Each lane then has at most two tasks in flight, one per tier. The opus worker
stops finding "no ready task" in a critical lane only because a sonnet refactor is in flight there.

## Context
- Observed 2026-11-02 at 02:20 UTC on origin/main 94e1e477. `make next-task TIER=opus` and the per-lane runs for
  engine, harness, research and functions all printed `no ready task`. With the busy rule ignored, the same board
  held 25 ready, unclaimed opus engine tasks, 22 harness and 28 research. That includes the M2 exit owner T-3324
  (engine, P2) and T-3584 (research, P2, which T-3445 waits on). M2's exit is down to 264 probe cells, all in
  probes/t3298-case-mapping-sweep, probes/t3276-collation-other-scripts and probes/t3334-criteria-operand-forms.
- What made each lane busy (`branch_claims` → `active` in scripts/tasks.py):
  - engine: `agent/engine/T-3439-split-reference-operators`, a `tier: sonnet` P3 file split. It hid every opus engine
    task, T-3324 included. The two tasks touch different files (reference_operators.rs vs
    functions/util/collation.rs).
  - harness: `agent/harness/T-3703-...` (opus) and `agent/harness/T-3674-...` (opus).
  - research: `agent/research/T-3571-...` (opus, M9).
- Each opus worker run that finds every lane busy launches the planner (routines.md, opus worker prompt), which files
  more tasks into the same hidden queues. This task is one such run's follow-up.
- Rules in play:
  - Decision 0022: one task in flight per lane, enforced by `available` in scripts/tasks.py. Its stated purpose is to
    keep parallel worker runs from colliding in one lane.
  - Decision 0018: there are two tier workers, a sonnet and an opus routine, staggered.
  - T-3558, the dependency, edits the same filter (the P0 bypass and the `lane ... busy` message). This task builds
    on it so the two PRs do not conflict.
- `effective_tier` already gives a branch's tier: handoffs and review-limited branches count as opus. `active` holds
  (lane, task id), so the busy set can become (lane, tier) with the tier taken from the task on main, or from the
  branch's task file when the task is not on main.
- Conflict risk, which the decision must weigh: two tasks in one lane at once may edit the same module. Measure it
  before deciding. Over the last 7 days of `git log --first-parent origin/main`, for pairs of merged PRs in one lane
  whose branches overlapped in time, count the pairs that touched a common non-task file, and put the count in the
  decision record.
- scripts/ and `.claude/` are protected paths, so the PR needs a decision record (`make new-decision`).
  docs/process/routines.md repeats the busy rule as prose in both worker prompts. Agents never edit routines, so the
  PR updates the routines.md text and adds a "Needs owner" line asking the owner to re-apply the prompts.

## Acceptance criteria
- [ ] A decision record states the per-tier busy rule and the conflict count above, and amends decision 0022 by
      reference.
- [ ] `python3 scripts/tasks.py next --lane engine --tier opus` offers a ready opus task when the lane's only
      in-flight branch is a sonnet task's, and `--tier sonnet` still offers nothing in that lane. Tests in
      scripts/test_tasks.py cover four cases: a sonnet branch with opus and sonnet candidates, the mirror case, a
      handed-off branch (counts as opus), and a lane with one branch of each tier (nothing offered).
- [ ] `ready` gives the same answer as `next` under the new rule, and so does T-3558's `lane ... busy` message: it
      names only branches of the asking tier.
- [ ] docs/process/agent-workflow.md and the two worker prompts in docs/process/routines.md describe the per-tier
      rule. docs/status.md "Needs owner" asks for the routines to be re-applied.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The P0 bypass and the `lane busy` message (T-3558).
- Letting current-milestone exit work pass a lane busy with later-milestone work (T-3824).
- The claim window, handoffs and review-limited branches (decisions 0018, 0095).
- More than one task in flight per tier and lane.
