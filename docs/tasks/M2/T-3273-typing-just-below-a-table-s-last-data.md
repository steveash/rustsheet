---
id: T-3273
title: "Typing just below a table's last data row grows the table by a row (references.md rule 7e, t3003-below-last-row)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3218]
---

## Goal
A formula or a constant typed into a cell just below a table's last data row, inside the table's columns, grows the
table by one row, as Excel does. `ROWS(T[#All])` and every structured reference to the table see the new row,
the typed cell keeps what was typed, and the facade's undo of that edit puts the table back.

## Context
Evidence: docs/excel-semantics/references.md "Calculated columns" rule 7(e) (T-3003, excel-graph 2026-10-30). Table
`T` A1:C4 with no totals row. `=B4*2` or the constant `7` typed into A5 makes `ROWS(T[#All])` 5 and `COLUMNS(T[#All])`
3. A5 keeps the formula `=B4*2` (FORMULATEXT, ISFORMULA) or the constant, and column D stays blank.

Cases owned (decision 0096; every failing cell of each, split from T-3218 at planning 2026-10-30):
- probes/t3003-below-last-row S!H2 (`=ROWS(T[#All])`: Excel 5, ours 4)
- probes/t3003-below-last-row-const S!H2 (the same readout with the constant `7` typed)

Every other cell of both cases already passes, so the typed value itself is right. Only the growth is missing.

Code:
- The engine has no row-growth path. `Model::table_expansion` and `Model::expand_table`
  (crates/rustsheet-engine/src/calculated_columns.rs) grow only by a column, and only for a formula.
- `Model::set_typed_user_input` (crates/rustsheet-engine/src/typed_input.rs) calls them.
- The facade's `Txn::log_typed_fill` (crates/rustsheet/src/edit/calculated_column.rs) logs the table for undo
  (`Undo::TableBefore`) and returns early for non-formula text.

T-3218 rewrites the same guards to let constants and a totals row grow a column (rules 7a and 7c), hence the
dependency: build the row path on its shape and keep one guard function for both directions. Do not copy it.

Size M, opus: this adds a second growth direction to the engine and the facade's undo log. Any cell typed below a
table now changes the table, so the existing typed-input tests and eval cases that type below a table must keep
passing.

## Acceptance criteria
- [ ] probes/t3003-below-last-row S!H2 and probes/t3003-below-last-row-const S!H2 pass bit-exact. Their F lines
      leave eval/baseline/probes.txt via `make eval-baseline`, with 0 regressions in every suite.
- [ ] Growth goes through the same engine entry point as the column growth (`set_typed_user_input` via
      `table_expansion` or its successor) for both typed formulas and typed constants. `set_user_input` (stored or
      untyped input) never grows a table, and a unit test shows that.
- [ ] Facade test: a `Workbook::edit` that types into the cell below a table grows it, and undoing that edit
      restores the original table reference (`T[#All]` back to 4 rows).
- [ ] Engine tests cover the unprobed neighbours and pin today's behavior for each, with a comment saying it is
      unprobed:
      - a table with a totals row;
      - a cell below the table but outside its columns;
      - a cell two rows below the table;
      - an overlap with another table.
- [ ] references.md rule 7(e)'s "Ours" sentence says what the engine now does. `make check` green; reviewers approve.

## Out of scope
- Filling a calculated column into the new row (unprobed: the probe's `Calc` column is empty and B5/C5 are not read
  back). File a probe task in research if the implementation needs that answer.
- Growth below a table with a totals row, and growth by several rows (pasting or typing further down): unprobed.
- Column growth (T-3218) and unqualified `[@Col]` beside a table (T-3274).
