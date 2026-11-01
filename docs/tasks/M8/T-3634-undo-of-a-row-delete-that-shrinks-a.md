---
id: T-3634
title: "Undo of a row delete that shrinks a table at its bottom restores the rows but not the table's extent"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Undoing a row delete that took a table's bottom data rows restores the table's extent along with the rows, so undo
leaves the workbook exactly as it was.

## Context
- Found by T-3259 (2026-11-01). `UserModel::delete_rows(0, 6, 2)` on the `edge_model` table `Sales` (B3:C7, data rows
  4-7) shrinks it to B3:C5; `undo()` re-inserts the rows and their cells but the table stays B3:C5. The undo of
  `Diff::DeleteRows` (`user_model/diff_undo.rs` `undo_delete_rows`) calls `Model::insert_rows` at the first deleted
  row, which is just below the table's remaining data, and an insert there does not grow a table (Excel's rule,
  probes/table-edits/table-insert-row-below). The same holds for T-3259's delete of every data row, which keeps row 4
  and deletes rows 5-7.
- History has no diff that records a table's extent; `Diff::DeleteSheet` carries the sheet's tables, the model to
  follow. A fix records the tables a row delete resized and restores them on undo (and redo re-applies them).

## Acceptance criteria
- [ ] Undo of a delete of a table's bottom data rows, and of T-3259's delete of every data row, restores the table's
      reference; redo shrinks it again. Unit tests in test_structured_references.rs (T-3259's test has a
      `TODO(T-3634)` for this assertion).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column deletes (check whether they share the gap; file a task if so).
