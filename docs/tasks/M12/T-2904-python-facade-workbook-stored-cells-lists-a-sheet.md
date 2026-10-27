---
id: T-2904
title: "Python facade: Workbook.stored_cells lists a sheet's stored cells as A1 strings"
milestone: M12
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Python callers can walk a sparse sheet without reading its whole used range:
`wb.stored_cells("Sheet1")` returns `["Sheet1!A1", "Sheet1!C7", ...]`, every stored cell in row order, as
`Workbook::stored_cells` does in Rust.

## Context
The Rust facade has `Workbook::stored_cells(sheet) -> Result<Vec<CellRef>, RefError>` (crates/rustsheet/src/
workbook.rs): every stored cell of the sheet (values, formulas and spill members), by row and within a row by
column, cheap for a sparse sheet whatever its `used_range` (`RefError::UnknownSheet` for an unknown sheet). The
Python `Workbook` (bindings/python/src/facade.rs) exposes `used_range` and `values`, but `values` over the used
range of a sheet with one cell at A1 and one at XFD1048576 hits the `max_read_cells` cap, so Python has no way to
list the cells a workbook actually holds.

Format each `CellRef` with `CellRef::to_a1_in(sheet_name)` (crates/rustsheet/src/refs.rs), exactly as
`used_range` formats its range with `RangeRef::to_a1_in` and `name_of(&sheet_names(wb), id)`; read with
`self.read(py, ...)` so the GIL is released, like the other readers.

## Acceptance criteria
- [ ] `Workbook.stored_cells(sheet: str) -> list[str]` returns the Rust order, formatted with `CellRef::to_a1_in`.
- [ ] bindings/python/tests/test_facade.py gains tests that show: (1) after setting `C7`, `A1` and a formula
      `=SEQUENCE(2)` in `E1`, `stored_cells("Sheet1")` is `["Sheet1!A1", "Sheet1!E1", "Sheet1!E2", "Sheet1!C7"]`
      (row order, spill member included); (2) a sheet with cells at A1 and XFD1048576 returns both without hitting
      the read cap; (3) an empty sheet returns `[]`; (4) a sheet name that needs quoting (`"My Sheet"`) gives
      `'My Sheet'!A1`; (5) an unknown sheet raises `rs.RefError`.
- [ ] The method has a doc comment (bindings/python/docs/facade_api_reference.rst renders it through
      `autoclass`) and a stub in bindings/python/python/rustsheet/_ironcalc.pyi (`class Workbook`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A per-cell info object (`Workbook::cell` / `CellInfo`): `value`, `formula` and `number_format` already cover it.
- Other missing mirrors (T-2899 to T-2903, T-0460, T-2054); any change to crates/rustsheet or the engine.
