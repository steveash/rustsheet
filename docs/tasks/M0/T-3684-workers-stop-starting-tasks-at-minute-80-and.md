---
id: T-3684
title: "Workers stop starting tasks at minute 80 and spend the rest landing their PRs"
milestone: M0
lane: quality
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A worker session leaves behind as few unmarked or ejected PRs as it can, and leaves facts with
each one it could not land.

## Context
Owner request, 2026-11-01. Workers start tasks until minute 110 (decision 0025), so late tasks are
still in CI or review when the session ends. At 07:05 UTC on 2026-11-01, 8 PRs were ejected from
the merge train and about 10 non-draft PRs were never marked ready, all idle for hours. A ready PR
is the merger's, so it is not the problem.

## Acceptance criteria
- [x] Both worker prompts in docs/process/routines.md start new tasks only until minute 80, then
      land their own PRs until minute 110.
- [x] The work-loop skill has a landing-window step (6.6): mark unmarked PRs, clear ejected ones,
      and write `## Landing notes` for any PR left unlanded.
- [x] Decision 0229 records the rule and amends 0025.
- [x] Reviewers approve (verdict records on #2736); CI green before marking.

## Out of scope
- Queued repair tasks for ejected PRs (a follow-up, which these landing notes feed).
- Task-only PRs' review requirement.

## Outcome
Worker prompts, routines.md, agent-workflow.md and the work-loop skill now stop new tasks at minute
80 and add step 6.6, the landing window (decision 0229, amending 0025). The owner re-applies the two
worker prompts. No code or eval change.
