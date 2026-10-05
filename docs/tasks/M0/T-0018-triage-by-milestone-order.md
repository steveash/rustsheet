---
id: T-0018
title: "Gap-triager and functions lane work in milestone order"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The functions lane spends its runs on the current milestone's functions, not on M9 last-digit tails.

## Context
2026-10-04: 12 of the functions lane's open tasks were M9 numeric tails (two attempts already
blocked: T-0901, T-0925) while M2's everyday functions had no oracle evidence yet. The
gap-triager ranks by failing cells × frequency only.

## Acceptance criteria
- [ ] gap-triager ranks groups by (milestone of the function, earliest in phase order first), then failing cells × frequency, and sets the task milestone from the catalog.
- [ ] `scripts/tasks.py next` orders ready tasks by milestone before priority, using the phase
  order in docs/milestones.md (decision 0026), not the milestone number, (or the functions routine prompt passes the current milestone); M9 tasks still get picked when nothing earlier is ready.
- [ ] M9 numeric-fitting tasks that need oracle sweeps say so and depend on a sweep task instead of being attempted blind.
- [ ] Decision record if protected paths change; `make check` green; reviewers approve.
