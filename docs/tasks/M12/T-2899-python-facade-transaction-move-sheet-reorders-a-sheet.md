---
id: T-2899
title: "Python facade: Transaction.move_sheet reorders a sheet by name"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers of the typed facade can reorder sheets: `with wb.edit() as tx: tx.move_sheet("Sheet3", 0)` moves
the sheet named `Sheet3` to the first tab position, as `Txn::move_sheet` does in Rust.

## Context
The Rust facade has `Txn::move_sheet(id, position)` (crates/rustsheet/src/edit.rs; 0-based tab position, 3-D
spans like `Sheet1:Sheet3!A1` follow the new order; `EditError::Ref` for an unknown sheet or a position past the
last sheet). The Python `Transaction` (bindings/python/src/facade.rs) only buffers `Set`, `AddSheet`,
`RenameSheet`, `DeleteSheet` and `Clock` ops (`enum Op`, applied by `apply`), so Python cannot move a sheet. This
is one of the Rust-facade methods with no Python mirror, like `Iteration`/data tables (T-0460) and the workbook
name (T-2054). It is not part of T-2882's legacy-surface question (that spec is about the raw `Model`/`UserModel`
classes).

The other sheet ops already address sheets by name through `find_sheet`, which also sees sheets added or renamed
earlier in the same block; the new op must do the same and keep the local `sheets` list in the new order, so a
later op in the block still resolves names.

## Acceptance criteria
- [ ] `Transaction.move_sheet(name: str, position: int) -> None` buffers an op that `apply` runs as
      `Txn::move_sheet`, resolving `name` with `find_sheet` and reordering the local `sheets` list to match.
- [ ] bindings/python/tests/test_facade.py gains tests that show: (1) after `tx.add_sheet("B"); tx.add_sheet("C")`
      and `tx.move_sheet("C", 0)`, `wb.sheet_names == ["C", "Sheet1", "B"]`; (2) a formula
      `=SUM(Sheet1:B!A1)` changes value when a sheet holding a number moves into or out of the 3-D span;
      (3) an unknown name and a position past the last sheet raise `rs.EditError` and leave the workbook
      unchanged (rollback, as `test_failing_op_rolls_back_whole_transaction` checks for other ops);
      (4) a `move_sheet` of a sheet added earlier in the same block works.
- [ ] The method has a doc comment (bindings/python/docs/facade_api_reference.rst renders it through
      `autoclass`) and a stub in bindings/python/python/rustsheet/_ironcalc.pyi (`class Transaction`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other missing mirrors (defined names T-2900, number formats T-2901, hidden rows T-2902, tables T-2903,
  stored cells T-2904, iteration/data tables T-0460, workbook name T-2054).
- Any change to crates/rustsheet or the engine.
