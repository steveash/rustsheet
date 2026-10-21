---
id: T-2284
title: "Tasks blocked at the review-round limit with fixes pushed get re-reviewed by a worker"
milestone: M5
lane: harness
status: done
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
- On 2026-10-21 four PRs sat in this state with the last round's fixes pushed and unreviewed (the owner's
  session then unblocked and merged all four by hand):
  #1357 (T-2194, the SPEC-0016 spec, which T-1699 #1368 also waits on), #1385 (T-2216/T-2217
  task filing), #1245 (T-1484, blocked only by the performance reviewer that decision 0081 has
  since turned off), and #1353 (T-1890, a real blocking finding).
- Distinguish "blocked: review-round limit, fixes pushed" from real blockers (time-boxed fits,
  decision 0056; Phase 2 dependencies; desktop oracle, decision 0039).

## Acceptance criteria
- [x] A machine-readable marker for "blocked at the review-round limit" in the task file (e.g. a
      `Blocked because: review-round limit` first line), written by work-loop step 5.
- [x] `make next-task` offers such a task to the opus worker (after handoffs, before new work),
      once the branch has been idle 2 h, and the work loop's resume step re-runs the full
      reviewer panel from round 1 on the current diff.
- [x] The steward lists these PRs under open PRs with their age.
- [x] The work-loop skill, scripts/tasks.py and docs/process/routines.md are protected paths: the PR
      carries a decision record, and the routine prompts change only through the owner (decision 0008).
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising the round limit.

## Outcome
Decision 0095. Work-loop step 5 now starts the "Blocked because" section of a task blocked only by
the round limit with `Blocked because: review-round limit`. `scripts/tasks.py branch_claims` returns
such tasks (on `status: blocked` branches idle 2 h) with their branch; `next`/`ready` offer them to
opus only, after P0 tasks and handoffs and before other work, whatever their lane is doing, and
print `resume <branch>: blocked at the review-round limit (work-loop step 5c)`. New work-loop step
5c resumes the branch and re-runs the full reviewer panel from round 1; a second limit writes
`Blocked because: review-round limit, twice`, which is not offered again and goes to the owner.
The steward skill lists these PRs with their age. Routine prompts are unchanged (decision 0008);
`next-task` and work-loop step 1 carry the case. Eval delta: none (process change).
