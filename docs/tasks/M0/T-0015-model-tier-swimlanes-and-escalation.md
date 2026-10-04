---
id: T-0015
title: "Model-tier swimlanes: `tier` task field, tier routines, escalation handoff"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
depends_on: []
---

## Goal
Cheaper models do the simple tasks and the expensive model does the hard ones, with a defined way
for a stuck cheaper run to hand a task up without losing work, so throughput per unit of usage rises.

## Context
Owner request (2026-10-04). Usage limits, not wall-clock, are the constraint. Measured prices per
MTok in/out: Haiku 4.5 $1/$5, Sonnet 5.5 $2/$10, Opus 5.5 $4/$20. Start with two tiers
(`sonnet`, `opus`); add `haiku` only if Sonnet-tier data shows a class of tasks Haiku could do.
Gates do not change by tier: `make check`, CI and every applicable reviewer must approve.

## Acceptance criteria
- [ ] Task front matter gains `tier: sonnet | opus` (default `opus`), validated by `scripts/tasks.py`; `make next-task TIER=<tier>` filters on it; docs/process/task-format.md documents the rubric (sonnet: size S, clear acceptance criteria, evidence already exists, no spec, no protected paths, no numeric fitting; else opus).
- [ ] Every open task gets a tier (planner/gap-triager assign it for new tasks).
- [ ] Escalation protocol in the work-loop skill: a sonnet run that fails `make check` twice on the same cause, gets REQUEST_CHANGES twice, or judges the task beyond it pushes its branch, appends a `## Handoff` section (what was tried, what failed, next step), sets `tier: opus`, and stops. Opus runs prefer handed-off tasks.
- [ ] Reviewer agents get explicit `model:` frontmatter: security, quality and architecture on Sonnet; Excel-fidelity and performance on Opus (decision record says why; revisit after a week).
- [ ] docs/process/routines.md defines a sonnet worker and an opus worker (replacing the functions/critical-lane pair; both use the critical-lane order, one task in flight per lane), and a sonnet run may take up to 3 tier-sonnet tasks per session.
- [ ] Decision record for the protected-path changes; `make check` green; reviewers approve.

## Out of scope
Changing gates; creating or editing routines (the owner updates them from routines.md).
