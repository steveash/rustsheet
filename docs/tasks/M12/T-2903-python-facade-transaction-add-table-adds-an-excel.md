---
id: T-2903
title: "Python facade: Transaction.add_table adds an Excel table over an A1 range"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers of the typed facade can add an Excel table (ListObject) so that structured references work:
`tx.add_table("Sheet1!A1:B4", "Sales", filtered=False, totals_row=False)` makes `=SUM(Sales[Qty])` evaluate, as
`Txn::add_table` and `Txn::add_table_with_totals` do in Rust.

## Context
The Rust facade has `Txn::add_table(range, name, filtered)` (T-1036) and `Txn::add_table_with_totals(range, name,
filtered, totals_row)` in crates/rustsheet/src/edit.rs. The first row of the range is the header row (set the
header cells first); with `filtered` the table carries an active filter, so on that sheet hidden rows count as
filtered and SUBTOTAL 1-11 skip them; with `totals_row` the last row is `Table[#Totals]`. Errors:
`EditError::Ref` for an unknown sheet, `EditError::Table` for an invalid or taken name, a range outside the sheet,
no data row, an overlap with another table, or an empty or repeated header. The Rust tests in
crates/rustsheet/src/edit/tests.rs (`table_with_totals_row_reads_totals_and_all`, `a_failed_edit_removes_the_table`
and the filtered-table SUBTOTAL test next to them) show the expected values.

The Python `Transaction` (bindings/python/src/facade.rs, `enum Op`, `apply`) has no table op. The Rust methods take
a `RangeRef`, so `apply` must turn the A1 string into one with `rustsheet::parse_range` (crates/rustsheet/src/refs.rs)
and a resolver over the block's local `sheets` list (`resolve_sheet_by_name`, as `find_sheet` uses), so that a sheet
added earlier in the same block can be named. Reuse that resolver; do not add a second sheet-name lookup.

## Acceptance criteria
- [ ] `Transaction.add_table(range: str, name: str, filtered: bool = False, totals_row: bool = False) -> None`
      buffers an op that `apply` runs as `Txn::add_table_with_totals(parse_range(...), name, filtered, totals_row)`.
- [ ] bindings/python/tests/test_facade.py gains tests that show: (1) with headers Name/Qty in A1:B1 and Qty
      values 1, 2, 4 in B2:B4, a table `Sales` over `Sheet1!A1:B4` makes `=SUM(Sales[Qty])` 7; (2) with
      `totals_row=True` over A1:B5, `=ROWS(Sales[#All])`, `=ROWS(Sales[#Data])` and `=ROWS(Sales[#Totals])` are 5, 3
      and 1; (3) a duplicate table name and an overlapping range raise `rs.EditError` and roll the block back (no
      table is left: a later `add_table` of the same name succeeds); (4) a table on a sheet added earlier in the
      same block works.
- [ ] The method has a doc comment (bindings/python/docs/facade_api_reference.rst renders it through
      `autoclass`) and a stub in bindings/python/python/rustsheet/_ironcalc.pyi (`class Transaction`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deleting or converting tables (`delete_table`, `convert_table_to_range` are behind the `harness_row_insert`
  feature of crates/rustsheet and not public API).
- Hidden rows (T-2902); other missing mirrors (T-2916, T-2900, T-2901, T-2904, T-0460, T-2054); any change to
  crates/rustsheet or the engine.
