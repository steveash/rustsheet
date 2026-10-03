---
id: T-0103
title: "API v0 write path: edit() transactions returning ChangeSet"
milestone: M1
lane: api
status: todo
priority: P1
size: M
depends_on: [T-0102]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Implement `Workbook::edit`/`set` with rollback on error and a ChangeSet of input and value changes (computed by diffing before/after a full recalc in v0).

## Context
Per SPEC-0001; incremental ChangeSet comes in M3.

## Acceptance criteria
- [ ] Atomic via undo log (no whole-model clone): a failing op inside edit leaves the workbook unchanged (test).
- [ ] Bench recorded: one-cell edit and rollback on >=100k formulas; `set_track_changes(false)` opt-out implemented.
- [ ] ChangeSet lists exactly the cells whose values changed (property test vs full diff).
- [ ] Eval spec adapter uses `edit`.
- [ ] `make check` green; reviewers approve.

## Out of scope
Undo/redo, structural edits.
