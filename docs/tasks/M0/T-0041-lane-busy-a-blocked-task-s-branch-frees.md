---
id: T-0041
title: "Lane busy: a blocked task's branch frees its lane"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A lane is busy only while its in-flight task can still progress. A branch whose task file is
`status: blocked` on that branch does not stop the lane's next task for 2 hours.

## Context
On 2026-10-05 the opus worker resumed the T-0125 handoff and then set it `status: blocked`
(waiting on T-0213) at 06:3x. The fresh commit kept the `engine` lane busy until about 08:40
(`BUSY_SECONDS`, decision 0022). So T-0105 (632 gen-m1 cells, the largest M1 gap) could not be
picked, and the opus queue fell through to M0 leftovers. `available()` already frees a lane whose
branch's task is `done` on main (T-0034). A blocked task is the same case, except that its
`status: blocked` lives on the branch, not on main.

Code: `scripts/tasks.py::branch_claims` already reads each agent branch's task file at the branch
tip (the `ls-tree`/`show` loop for `## Handoff`). Read its `status:` there with the existing
front-matter parser (`parse`, not a new regex; directive 3), and leave blocked tasks out of
`active`. A branch that is blocked and also has an unresumed `## Handoff` is already outside
`active`, so the new test only needs the plain-blocked case. Keep blocked
tasks claimed, so nobody restarts them: unblocking is decision 0013's job.

## Acceptance criteria
- [x] A task whose pushed branch sets `status: blocked` does not make its lane busy, but stays
  claimed. Unit test in `scripts/test_tasks.py`, next to the done-task test from T-0034.
- [x] Decision 0022's busy-lane wording and docs/process/agent-workflow.md mention the exception.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `BUSY_SECONDS` or the claim TTL.

## Outcome
`scripts/tasks.py::branch_claims` now reads each agent branch's task front matter (via
`front_matter`, split out of `parse`) and leaves branches whose task is `status: blocked` out of
`active`, so a blocked task no longer keeps its lane busy; it stays claimed. Unit test
`test_blocked_task_on_its_branch_frees_the_lane_but_stays_claimed`; decision 0022 and
agent-workflow.md document the exception. Eval delta: none (scripts only).
