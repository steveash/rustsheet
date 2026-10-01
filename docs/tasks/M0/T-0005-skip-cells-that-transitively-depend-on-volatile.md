---
id: T-0005
title: "Skip cells that transitively depend on volatile functions; inject a fixed clock"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
Avoid false failures from cells that depend on NOW/TODAY/RAND indirectly, and make TODAY-based cases testable.

## Context
runner.rs only skips cells whose own formula is volatile. rustsheet-xlsx has a `mock_time` feature that reads a clock from a METADATA sheet.

## Acceptance criteria
- [ ] Expectation filtering marks cells whose precedents (static reference analysis is sufficient) include a volatile cell as `skipped_volatile`.
- [ ] When a workbook has IronCalc's METADATA mock-time convention, the engine adapter evaluates with that clock and volatile date cells are scored.
- [ ] Tests cover both; eval baseline unchanged or improved.
- [ ] `make check` green; reviewers approve.

## Out of scope
General dependency graph (M3).
