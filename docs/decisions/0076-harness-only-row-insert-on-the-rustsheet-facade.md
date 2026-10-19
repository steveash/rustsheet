# 0076 — Harness-only row insert on the rustsheet facade

- **Status:** accepted (2026-10-19, T-1933)
- **Context:** T-1933 adds a row-insert sheet op to eval cases so that T-1853's result (a legacy
  cell stays legacy after a row insert) is an oracle-filled probe. The eval engine adapter edits
  workbooks only through the `rustsheet` facade, whose `Txn` has no row insert, and the engine
  `Model` is `pub(crate)` there. A public `Txn::insert_rows` would need undo and change sets for a
  structural edit and an amendment to SPEC-0001: API work of its own.
- **Decision:** the facade gets `Workbook::insert_rows(sheet, row, count)` behind the cargo feature
  `harness_row_insert`, enabled only by `rustsheet-eval`, as `metadata_clock` gates
  `pin_clock_from_metadata` (SPEC-0008). It calls the engine's `Model::insert_rows`, is not a
  `Txn` edit (no undo, no change set, no recalculation) and is not part of the public API.
- **Consequences:** eval cases can replay `{"op":"insert_row",...}` on both the oracle and the
  engine side. A public structural-edit API, when it comes, replaces this method, and the adapter
  then moves to it; until then no non-harness caller may enable the feature.
