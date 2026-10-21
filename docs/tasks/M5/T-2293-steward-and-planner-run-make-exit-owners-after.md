---
id: T-2293
title: "Steward and planner run make exit-owners after make eval (decision 0096)"
milestone: M5
lane: quality
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2259]
---

## Goal
The steward's daily run and the planner's milestone run use `make exit-owners MILESTONE=<current>`
(decision 0096) to find failing exit cells with no open owner, instead of a hand search of
`docs/tasks/`, and give each listed cell an owner.

## Context
- T-2259 added `scripts/exit_owners.py` and `make exit-owners`; changing `.claude/` was out of its
  scope. On 2026-10-21 the planner found 13 unowned M5 exit cells by hand (decision 0096 Context).
- `.claude/` is a protected path (`scripts/check_governance.py`), so the PR amends decision 0096 or
  adds a record, hence tier opus.

## Acceptance criteria
- [ ] `.claude/skills/steward/SKILL.md` (health run, after `make eval`) and `.claude/agents/planner.md`
      (milestone planning) run `make exit-owners MILESTONE=<current milestone>` and say what to do
      with each listed cell: add it to the open task that should own it, or file one.
- [ ] A decision record (or an amendment of 0096) covers the protected-path change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making `make exit-owners --strict` a CI gate.
