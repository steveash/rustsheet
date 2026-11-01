---
id: T-3703
title: "Push review-round fixes once per round, not per finding"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Agents push the fixes of a review round once, after every reviewer of that round has reported,
so a round costs one CI run instead of one per fix.

## Context
Owner request (2026-11-01, CI-speed investigation). On 2026-11-01 (05:53-09:16Z) 20 of 78 PR CI
runs were cancelled because a newer push superseded them (about 77 runner-minutes); PRs averaged
1.9 runs, up to 5. The review skill posts verdict records per reviewer as each returns; fixes can
then be pushed piecemeal.

## Acceptance criteria
- [x] The `review` and `work-loop` skills say to collect all of a round's findings, fix them in
      one or more local commits, run `make check`, and push once per round (verdict records still
      name the reviewed head, as decision 0178 requires).
- [x] A decision record (protected path `.claude/`); `make check` green; reviewers approve (round 1 on b246a9681: all three APPROVE with MINOR findings, fixed in round 2).

## Out of scope
- Changing the review-round limit or the panel (T-3702).

## Outcome
Decision 0240 adds "Push once per round" to the `review` skill (step 4), and work-loop step 5
points to it. A worker waits until every reviewer of the round has reported, fixes all of the
round's findings locally, runs `make check`, and pushes once.

Baseline for later measurement (2026-11-01, 05:53-09:16Z): 20 of 78 PR runs were cancelled
because a newer push superseded them, and PRs averaged 1.9 runs.

Stacked on T-3702 (#2740), which also edits the review skill.
