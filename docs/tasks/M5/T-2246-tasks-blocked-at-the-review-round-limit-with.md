---
id: T-2246
title: "Tasks blocked at the review-round limit with fixes pushed get re-reviewed by a worker"
milestone: M5
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A task blocked only because the work loop's 3 review rounds ran out gets picked up again. Its
fixes are already pushed, so a later run re-reviews the branch and merges it if every reviewer
approves, instead of the branch staying claimed forever.

## Context
- Work-loop step 5 allows three review rounds. After that the run sets `status: blocked` on the
  branch, marks the PR draft and moves on. `tasks.py branch_claims` counts a task blocked on its
  branch as claimed at any age (decision 0048). Both worker prompts resume only branches that are
  *not* `status: blocked`. The steward's step (4) only merges non-draft, non-blocked PRs. So no
  routine ever re-reviews these branches.
- On 2026-10-21 four PRs sat in this state with the last round's fixes pushed and unreviewed:
  #1357 (T-2194, the SPEC-0016 spec, which T-1699 #1368 also waits on), #1385 (T-2216/T-2217
  task filing), #1245 (T-1484, blocked only by the performance reviewer that decision 0081 has
  since turned off), and #1353 (T-1890, a real blocking finding).
- Distinguish "blocked: review-round limit, fixes pushed" from real blockers (time-boxed fits,
  decision 0056; Phase 2 dependencies; desktop oracle, decision 0039).

## Acceptance criteria
- [ ] A machine-readable marker for "blocked at the review-round limit" in the task file (e.g. a
      `Blocked because: review-round limit` first line), written by work-loop step 5.
- [ ] `make next-task` offers such a task to the opus worker (after handoffs, before new work),
      once the branch has been idle 2 h, and the work loop's resume step re-runs the full
      reviewer panel from round 1 on the current diff.
- [ ] The steward lists these PRs under open PRs with their age.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising the round limit.
