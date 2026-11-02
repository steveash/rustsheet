---
id: T-3824
title: "make next-task offers a current-milestone task over a lane busy only with later-milestone work"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3558, T-3821]
---

## Goal
A ready task of the current milestone is offered even when its lane is busy, as long as every branch making that lane
busy is working a task of a later milestone in the phase order. The current milestone's exit work then never waits for
an unrelated later-milestone task in the same lane to merge. A lane still holds at most one current-milestone task in
flight per tier, so the bypass adds at most one branch per lane and tier.

## Context
- Observed 2026-11-02 at 02:20 UTC on origin/main 94e1e477, with M2 current (docs/status.md snapshot heading). The
  research lane was busy with `agent/research/T-3571-bfrac-fraction-arithmetic`, an M9 opus task. That hid every
  research task, including T-3584 (M2, opus, ready). T-3584 is the probe that T-3445 (engine) waits on, and
  T-3445 owns 55 of M2's 264 remaining exit cells (probes/t3298-case-mapping-sweep Calc!C and D). The engine lane
  shows the same pattern whenever its in-flight branch is an M7-M9 task, as with T-3382 on 2026-11-01.
- M2's exit needs only T-3324, T-3445 (after T-3584), T-3583 and T-3404 (`make exit-owners MILESTONE=M2` reports 0
  unowned cells after `make eval` on 94e1e477). Each one that waits behind a later-milestone branch delays the exit by
  that branch's whole run.
- Inputs, one definition each (CLAUDE.md directive 3):
  - The current milestone: the `(M<n> is current)` in docs/status.md's `## Snapshot` heading, which the steward
    rewrites every run. With no such heading the bypass is off, which is today's behaviour.
  - The phase order: `PHASE1_MILESTONES` in scripts/phase1_exit.py, and the table in docs/milestones.md "Phase order"
    for the later phases. Reuse or move the tuple; do not copy it. Today `available` sorts by milestone number
    (`[1:].zfill(3)`), which is not the phase order: M3 and M4 sort before M5.
  - M0 (process and harness infrastructure) counts as earlier than every milestone, so an M0 branch is never bypassed.
- The busy filter is in `available` in scripts/tasks.py. T-3558 (P0 bypass, `lane ... busy` message) and T-3821
  (busy per tier) change it first; this task builds on both.
- scripts/ is a protected path, so the PR needs a decision record (`make new-decision`) that amends decision 0022 and
  says why a second branch in the lane is acceptable here: different milestone, different module, as in the T-3571 /
  T-3584 example.

## Acceptance criteria
- [ ] A decision record states the rule and its inputs, and amends decision 0022 by reference.
- [ ] With M2 current, `python3 scripts/tasks.py next --lane research --tier opus` offers a ready M2 research task
      when the lane's only in-flight opus branch is an M9 task's. Tests in scripts/test_tasks.py cover five cases:
      a later-milestone branch with current-milestone and later-milestone candidates (only the current one is
      offered), an M0 branch (nothing offered), a current-milestone branch (nothing offered), a missing snapshot
      heading (nothing offered), and the bypass already taken by a second branch (nothing offered).
- [ ] `ready` matches `next`. T-3558's `lane ... busy` message is printed only when no bypass applies.
- [ ] docs/process/agent-workflow.md and the worker prompts' busy-lane sentence in docs/process/routines.md describe
      the rule, and docs/status.md "Needs owner" asks for the routines to be re-applied (or extends T-3821's line if it
      is still open).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ordering candidates by phase instead of milestone number in general (file a follow-up if the decision wants it).
- The steward's critical-lane line and how it is computed.
- P0 tasks (T-3558) and the tier split (T-3821).
