---
id: T-3259
title: "One delete of every data row of a table keeps one cleared data row, as four one-row deletes do"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
probes/table-edits/table-delete-rows-all-data-once passes: one `delete_rows` of every data row of a table keeps
the table's first data row, cleared, exactly as four one-row deletes do (table-delete-row-all-data-below).

## Context
- Evidence (excel-graph 2026-10-30, T-3019): deleting rows 4-7 of `Sales` on S!B3:C7 in one edit leaves
  `ROWS(Sales)` 1, `ROWS(Sales[#All])` 2, `SUM(Sales[Qty])` 0, `COUNTA(Sales[Region])` 0, `ROW(Sales[#Headers])` 3,
  `=SUM(S!C4:C7)` rewritten to `=SUM(S!C4:C4)` (0), and the cells below move three rows, not four (`=S!C10` reads
  `=S!C7`, `=SUM(S!C9:C10)` reads `=SUM(S!C6:C7)`). docs/excel-semantics/references.md "Structured references under
  row deletes".
- The engine deletes all four rows: F!A2, A4, A5 (`#REF!`), A6 (1), A7 (`#REF!`), A8 (`=SUM(S!#REF!)`), A9 (`#REF!`)
  and A10 (1) fail (8 cells in eval/baseline/probes.txt). The one-row path that keeps the last data row is in
  crates/rustsheet-engine/src/table_edits.rs (T-2338, ~line 385) and user_model/common.rs ~1117; a multi-row delete
  covering every data row should take the same branch for its first row and delete the rest.
- The harness already models the kept row (`moved_input` in crates/rustsheet-eval/src/spill_extent.rs, T-3019).

## Acceptance criteria
- [ ] The 8 cells pass and leave eval/baseline/probes.txt; no new known failure.
- [ ] A unit test deletes every data row in one call and checks the kept, cleared row and the shift below.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A multi-row delete that covers every data row and more rows outside the table (unprobed).

## Outcome
Done 2026-11-01. `Model::kept_data_row` (table_edits.rs, renamed from `last_data_row`) now matches a row delete that covers exactly every data row of
a table under a header row, not only a one-row delete of its only data row: `Model::delete_rows` clears the first data
row's table cells and deletes the other rows (actions.rs), and `UserModel::delete_rows` records the clear and the
shorter delete as one history step (`range_clear_contents_diffs` and `delete_rows_diffs`, split out of the public
functions).
- probes/table-edits/table-delete-rows-all-data-once: 11/11 (the 8 known failures leave eval/baseline/probes.txt);
  every gated suite otherwise unchanged, no regressions. This lifts ROWS and COUNTA, which each had this case as their
  only visibly-off attributed cells, to the Phase 1 rate.
- Both paths refuse a bad delete before any change (`Model::check_delete_rows`, shared), so the user model never leaves
  an unrecorded clear: unit test `a_refused_delete_of_every_data_row_changes_nothing` (an array formula across the
  delete's end).
- A multi-row delete of every data row of a table with a totals row, or across a table beside it, takes the same rule
  unprobed (the one-row cases are probed: table-delete-row-only-data-totals, -outside, -second-table).
- Unit test `deleting_every_data_row_at_once_keeps_one_cleared_row`; `row_deletes_keep_values` now expects the table to
  follow a delete of every data row.
- Follow-up T-3634: undo restores the rows but not the table's extent, for this delete and for any delete of a table's
  bottom data rows (found here; pre-existing).

