---
id: T-0102
title: "Implement API v0 facade (read path + typed values)"
milestone: M1
lane: api
status: todo
priority: P1
size: M
depends_on: [T-0100, T-0101, T-0113]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Create crate `crates/rustsheet` with Workbook open/new/save, SheetId, CellRef/RangeRef parsing, `value`/`values`/`cell`, typed `Value` with `ErrorKind` (no string-flattened errors).

## Context
Per SPEC-0001. The eval harness engine adapter should switch to the facade once available.

## Acceptance criteria
- [ ] Facade crate with docs on all public items (`missing_docs` deny).
- [ ] Eval adapter uses the facade; eval results unchanged.
- [ ] Unit tests for reference parsing and value conversion.
- [ ] `make check` green; reviewers approve.

## Out of scope
Edits/ChangeSet (T-0103).
