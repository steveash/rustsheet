---
id: T-0100
title: "SPEC-0001: public API v0 (facade crate `rustsheet`)"
milestone: M1
lane: api
status: in-progress
priority: P0
size: M
depends_on: []
---

## Goal
Write the spec for the first public API, condensed from docs/architecture/api-design.md, scoped to what v0 implements over the current Model.

## Context
IronCalc exposes Model<'a>/UserModel with Result<_, String> and positional sheet indexes. API v0 must be FFI-friendly (decision 0004).

## Acceptance criteria
- [ ] docs/specs/SPEC-0001-public-api-v0.md: crate layout, types (Workbook, SheetId, CellRef, RangeRef, Value, XlError/ErrorKind, Input, ChangeSet, error enums), methods, Python mapping, test plan.
- [ ] Lists the implementation tasks (each S/M) and updates docs/tasks accordingly (planner).
- [ ] `make check` green; reviewers approve.

## Out of scope
Implementation.
