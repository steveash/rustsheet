---
id: T-2900
title: "Python facade: Transaction.define_name for workbook- and sheet-scoped defined names"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers of the typed facade can add defined names: `tx.define_name("Rate", "=Sheet1!$B$1")` adds a
workbook-scoped name and `tx.define_name("Local", "=$A$1", sheet="Sheet2")` a name scoped to one sheet, as
`Txn::define_name` and `Txn::define_sheet_name` do in Rust.

## Context
The Rust facade has `Txn::define_name(name, formula)` (T-0043) and `Txn::define_sheet_name(sheet, name, formula)`
(T-1193) in crates/rustsheet/src/edit.rs: the formula may have a leading `=` or not, a sheet-scoped name shadows a
workbook name of the same spelling on its sheet, and both give `EditError::DefinedName` for an invalid or duplicate
name or a formula that does not parse (`EditError::Ref` for an unknown sheet). The Python `Transaction`
(bindings/python/src/facade.rs, `enum Op` and `apply`) has no defined-name op, so Python can only get names by
loading an xlsx. `Txn::load_defined_name` is the importer's entry point (it accepts cell-like spellings) and is
not mirrored.

Sheet ops in `apply` resolve sheet names through `find_sheet` so that a sheet added earlier in the same block can
be named; the `sheet=` argument must do the same.

## Acceptance criteria
- [ ] `Transaction.define_name(name: str, formula: str, sheet: str | None = None) -> None` buffers an op; `apply`
      calls `Txn::define_name` when `sheet` is `None` and `Txn::define_sheet_name` with the `find_sheet` id
      otherwise.
- [ ] bindings/python/tests/test_facade.py gains tests that show: (1) `=Rate*2` on any sheet reads a workbook
      name; (2) a sheet-scoped `Rate` on Sheet2 shadows the workbook `Rate` for a formula on Sheet2 and not for one
      on Sheet1; (3) a duplicate name, an invalid name (`"1abc"`) and an unparsable formula each raise
      `rs.EditError` and roll the block back; (4) an unknown `sheet=` raises `rs.EditError`; (5) a name scoped to a
      sheet added earlier in the same block works.
- [ ] The method has a doc comment (bindings/python/docs/facade_api_reference.rst renders it through
      `autoclass`) and a stub in bindings/python/python/rustsheet/_ironcalc.pyi (`class Transaction`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reading or deleting defined names from Python (the Rust facade has no read API for names yet).
- `Txn::load_defined_name` (importer only).
- Other missing mirrors (T-2916, T-2901 to T-2904, T-0460, T-2054); any change to crates/rustsheet or the engine.
