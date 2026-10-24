# 0098 — Harness-only row delete, column insert and delete and table removal on the rustsheet facade

- **Status:** accepted (2026-10-21, T-2323)
- **Context:** T-2323 extends the case sheet ops (T-1608, T-1933) so that probes can delete rows,
  insert and delete columns and delete a table, the edits T-2197 could not probe. The eval engine
  adapter edits workbooks only through the `rustsheet` facade, whose `Txn` has none of these
  structural edits (decision 0076 gave the same reason for the row insert), and the engine `Model`
  is `pub(crate)` there.
- **Decision:** the facade gets `Workbook::delete_rows`, `insert_columns`, `delete_columns` and
  `remove_table` behind the existing cargo feature `harness_row_insert`, enabled only by
  `rustsheet-eval`, on the terms of decision 0076: they call the engine `Model`'s methods, are not
  `Txn` edits (no undo, no change set, no recalculation) and are not part of the public API. Sheet
  deletion already is a `Txn` edit (`Txn::delete_sheet`) and needs no new method.
- **Consequences:** eval cases can replay `delete_row`, `insert_column`, `delete_column`,
  `delete_sheet` and `delete_table` on both the oracle and the engine side. The engine has no
  table convert-to-range, so `convert_table` is refused by the adapter until an engine task adds
  it. A public structural-edit API, when it comes, replaces these methods; until then no
  non-harness caller may enable the feature.
