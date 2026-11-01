---
id: T-3703
title: "Push review-round fixes once per round, not per finding"
milestone: M0
lane: harness
status: in-progress
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
- [ ] The `review` and `work-loop` skills say to collect all of a round's findings, fix them in
      one or more local commits, run `make check`, and push once per round (verdict records still
      name the reviewed head, as decision 0178 requires).
- [ ] A decision record (protected path `.claude/`); `make check` green; reviewers approve.

## Out of scope
- Changing the review-round limit or the panel (T-3702).
