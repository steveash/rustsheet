---
id: T-2902
title: "Python facade: Transaction.set_row_hidden hides or shows a row"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers of the typed facade can hide and show rows: `tx.set_row_hidden("Sheet1", 3, True)` hides row 3
(the row of `Sheet1!A3`), so `SUBTOTAL(109, ...)` and AGGREGATE's ignore-hidden options skip it, as
`Txn::set_row_hidden` does in Rust.

## Context
The Rust facade has `Txn::set_row_hidden(sheet, row, hidden)` (T-1168, crates/rustsheet/src/edit.rs). Its `row`
is **0-based**; it gives `EditError::Ref` for an unknown sheet or a row outside the Excel limits. The Python
`Transaction` (bindings/python/src/facade.rs, `enum Op`, `apply`) has no such op.

Row numbering in Python: the facade's Python surface addresses cells with A1 strings, and the legacy Python
`Model.set_row_hidden` (bindings/python/src/raw_model.rs) already takes the 1-based row Excel shows. The Python
method therefore takes the **1-based** row number (1..=1,048,576) and converts it (`row - 1`) before calling the
Rust method; 0, a negative row or a row past 1,048,576 raises `rs.EditError` (convert with a checked subtraction,
never a wrapping cast). The sheet is named and resolved with `find_sheet`, like the other sheet ops, so a sheet added
earlier in the same block can be named.

## Acceptance criteria
- [ ] `Transaction.set_row_hidden(sheet: str, row: int, hidden: bool = True) -> None` buffers an op that `apply`
      runs as `Txn::set_row_hidden(id, row - 1, hidden)` with the `find_sheet` id.
- [ ] bindings/python/tests/test_facade.py gains tests that show: (1) with A1:A3 = 1, 2, 4 and
      `=SUBTOTAL(109,A1:A3)` in B1, hiding row 2 makes B1 5 and showing it again makes B1 7, while
      `=SUBTOTAL(9,A1:A3)` stays 7 (no filtered table on the sheet); (2) rows 0 and 1048577 and an unknown sheet raise
      `rs.EditError` and roll the block back; (3) row 1048576 is accepted.
- [ ] The method has a doc comment saying the row is 1-based (bindings/python/docs/facade_api_reference.rst renders
      it through `autoclass`) and a stub in bindings/python/python/rustsheet/_ironcalc.pyi (`class Transaction`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Hidden columns and reading the hidden flag (the Rust facade has neither).
- Other missing mirrors (T-2899 to T-2901, T-2903, T-2904, T-0460, T-2054); any change to crates/rustsheet or
  the engine.
