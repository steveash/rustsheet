---
id: T-0102
title: "Implement API v0 facade (read path + typed values)"
milestone: M1
lane: api
status: in-progress
priority: P1
size: M
depends_on: [T-0100, T-0101, T-0113, T-0114, T-0115]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Wire the facade `Workbook` (open/new/save, `value`/`values`/`cell`, `used_range`) onto the engine `Model`, using the ref types (T-0114) and `Value` conversion (T-0115) already in crate `rustsheet`.

## Context
Per SPEC-0001. The eval harness engine adapter should switch to the facade once available.

## Acceptance criteria
- [ ] Facade crate with docs on all public items (`missing_docs` deny).
- [ ] Eval adapter uses the facade; eval results unchanged.
- [ ] Unit tests for reference parsing and value conversion.
- [ ] `make check` green; reviewers approve.

- [ ] Remove the `expect(dead_code)` on `Value::from_cell` (T-0115) and the eval adapter's duplicate cell-to-value mapping.

## Out of scope
Edits/ChangeSet (T-0103).
