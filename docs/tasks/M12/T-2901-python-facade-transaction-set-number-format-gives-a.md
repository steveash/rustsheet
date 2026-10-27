---
id: T-2901
title: "Python facade: Transaction.set_number_format gives a cell a number format"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers of the typed facade can set a cell's number format: `tx.set_number_format("Sheet1!A1", "0.0%")`,
after which `wb.formatted("Sheet1!A1")` shows the value through it, as `Txn::set_number_format` does in Rust.

## Context
The Rust facade has `Txn::set_number_format(target, format)` (crates/rustsheet/src/edit.rs): it keeps the cell's
value and the rest of its style, changes no value, is undone with the transaction, and gives `EditError::Ref` for
an unknown sheet, a target outside the sheet or a multi-cell target (`EditError::Engine` if the engine rejects the
style). The Python `Workbook` can already read formats (`number_format`, `formatted` in
bindings/python/src/facade.rs) but the `Transaction` (`enum Op`, `apply`) cannot write one, so Python can only get
a format by loading an xlsx. The cell ops pass their A1 string straight to the `Txn` (`Op::Set` calls
`txn.set(r.as_str(), ...)`), so a sheet added earlier in the same block resolves; the new op does the same.

## Acceptance criteria
- [ ] `Transaction.set_number_format(ref: str, format: str) -> None` buffers an op that `apply` runs as
      `txn.set_number_format(ref.as_str(), &format)`.
- [ ] bindings/python/tests/test_facade.py gains tests that show: (1) a cell holding 0.5 with format `"0.0%"` has
      `wb.formatted(...) == "50.0%"`, `wb.number_format(...) == "0.0%"` and still `wb[...] == 0.5`; (2) a multi-cell
      target (`"Sheet1!A1:B2"`) and an unknown sheet raise `rs.EditError` and roll back every op of the block (the
      cell's earlier format and value stay); (3) `wb.changes["values"]` is empty after a format-only edit (the
      value did not change).
- [ ] The method has a doc comment (bindings/python/docs/facade_api_reference.rst renders it through
      `autoclass`) and a stub in bindings/python/python/rustsheet/_ironcalc.pyi (`class Transaction`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other style properties (fonts, fills, borders): SPEC-0001 non-goals; T-2882 decides their Python future.
- Other missing mirrors (T-2899, T-2900, T-2902 to T-2904, T-0460, T-2054); any change to crates/rustsheet or
  the engine.
