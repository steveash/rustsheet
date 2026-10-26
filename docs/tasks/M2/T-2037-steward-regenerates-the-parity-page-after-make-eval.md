---
id: T-2037
title: "Steward regenerates the parity page after make eval (owner-approved routine change)"
milestone: M2
lane: harness
status: dropped
priority: P3
size: S
tier: sonnet
depends_on: [T-2041]
---

## Goal
The daily steward regenerates docs/parity.md and docs/parity/ with `make parity-report` after its
`make eval`, so the linked parity page stays recent without a separate task each time.

## Context
T-2041 added `make parity-report` (decision 0079). The steward routine prompt
(docs/process/routines.md, the steward block) runs `make eval` but not the generator. Routine
prompts are owner-managed: agents never create, modify or delete routines, so the owner must
change the routine and its copy in docs/process/routines.md. Decision 0079 says the pages are
committed snapshots with no CI freshness gate; that stays true.

## Acceptance criteria
- [ ] The steward prompt in docs/process/routines.md runs `make parity-report` after `make eval`
  (owner change, with a decision record, since docs/process/ is protected).
- [ ] The next steward PR carries regenerated docs/parity.md and docs/parity/ files.

## Out of scope
- A CI freshness gate on the parity pages (decision 0079 rules it out).

## Outcome (dropped, 2026-10-25, planner)
Superseded by T-0076 (SPEC-0022). The parity page becomes the daily progress dashboard
(docs/progress.md, `make progress-report`). T-0076 carries the owner-approved steward routine
change, with its exact new prompt text, and is blocked on the owner as this task was.
