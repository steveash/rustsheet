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
- [ ] Atomic: a failing op inside edit leaves the workbook unchanged (test).
- [ ] ChangeSet lists exactly the cells whose values changed (property test vs full diff).
- [ ] Eval spec adapter uses `edit`.
- [ ] `make check` green; reviewers approve.

## Out of scope
Undo/redo, structural edits.
