---
id: T-2054
title: "Python bindings: expose the workbook file name (WorkbookOptions::name)"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers can name a workbook (`rustsheet.Workbook(name="Budget")`, and on load) so that
`CELL("address")` of a reference on another sheet and `CELL("filename")` give `[Budget.xlsx]...`.

## Context
T-2042 added `WorkbookOptions::name` to the facade (crates/rustsheet/src/workbook.rs); `None` is an unnamed
workbook, for which both CELL forms are `#N/IMPL!`. bindings/python/src/facade.rs builds `WorkbookOptions` with
`..WorkbookOptions::default()` (the `new` and `load_options` paths), so Python cannot set it yet.

## Acceptance criteria
- [ ] An optional `name` keyword on workbook creation and load reaches `WorkbookOptions::name`.
- [ ] A Python test shows `CELL("address",Data!B2)` is `[Budget.xlsx]Data!$B$2` for a named workbook and an
      error for an unnamed one; the stub/type hints are updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
