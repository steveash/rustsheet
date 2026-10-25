---
id: T-0065
title: "Adopt decision 0114: Phase 1 exits on fifteen-digit parity; semantics, accuracy, visible campaign stages; defer phase-1b"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The repository works to decision 0114: Phase 1 exits on fifteen-digit parity, `next` picks
`semantics` tasks, then `accuracy`, then `visible`, parked last-bit tasks are never picked, and the
eval report measures the new exit.

## Context
Owner decision of 2026-10-25 (decision 0114), from the Phase 1 assessment of `main` 2c52346.

## Acceptance criteria
- [ ] docs/milestones.md states the Phase 1 exit of decision 0114 and adds Phase 1b.
- [ ] `scripts/tasks.py` accepts `campaign: semantics | accuracy | visible` and orders them in that
      stage order; `defer: phase-1b` tasks are never ready; `scripts/test_tasks.py` covers both.
- [ ] docs/process/task-format.md and the gap-triager describe the stages and the defer key.
- [ ] The eval report shows per-function within-15-digit rates and the Phase 1 line
      (`v15_checkpoint`), and the milestone exit rows count visibly-off cells.
- [ ] The semantics and accuracy tasks of the assessment are filed or retagged; last-digit tasks
      carry `defer: phase-1b`; FORECAST.ETS value work moves to M14.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine or function change: the filed tasks do that.
- Lowering a gate or a baseline.
