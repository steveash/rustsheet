---
id: T-2896
title: "UserModel::apply_external_diffs refuses crafted diffs instead of panicking"
milestone: M12
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2913]
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
`UserModel::apply_external_diffs` returns `Err` for crafted `QueueDiffs` bytes (dangling sheet, style or row/column
indices, out-of-range cells) and leaves the model unchanged, instead of panicking.

## Context
SPEC-0026 decision 4: diff bytes come from a peer, so they are untrusted under directive 5.
`UserModel::apply_external_diffs` (crates/rustsheet-engine/src/user_model/common.rs) decodes `Vec<QueueDiffs>` with
bitcode and applies each `Diff` as it is. A diff that names a style index or sheet the model does not have reaches the
same unchecked indexing as T-2913's reproductions. Validate each decoded diff against the model before applying any
(reuse T-2913's index checks), so a refused list changes nothing. Write the first test the way T-2882 reproduced the
load panics: flush a real send queue, change one index in the bytes, apply. The Python binding drops this entry point
(T-2897); this task hardens the Rust API, which the web UI layer still uses.

## Acceptance criteria
- [x] A crafted diff list with a dangling style index and one with a dangling sheet index each get `Err`, with no panic
      and the model unchanged; the existing diff tests still pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- A fuzz target for diffs (file one if this task finds more than the shapes above).

## Outcome
A temporary probe of 57 crafted single-diff lists, followed by the security review, found three kinds of failure. All
three are now handled.

- **Diff data written into the sheet without a check.** Undo restores the cells, row and column records and sheets
  that the diff carries, and `SetMergedCells` installs ranges. Each is now checked with T-2913's checks, which
  `workbook_check.rs` exposes as `check_cell`, `check_row_record`, `check_style`, `check_merged_cells` and
  `check_worksheet`. The check runs against the model as it is when the diff applies. These undo bodies moved to
  `user_model/diff_undo.rs`, where the matrices are read with `get`.
- **`i32` overflow.** These were in the structural actions (`insert_rows`, `delete_rows` and their column versions, and
  `move_rows_action` and `move_columns_action`), in `set_user_array_formula`, in the range clears and in
  `Worksheet::set_row_style`. Each one now returns `Err` for a count, span or area off the grid. The checks are
  `expressions/utils` `is_valid_block`, `check_block` and `moved_span_in_grid`. These are public methods, so a direct
  call no longer panics either.
- **Work proportional to the area, not the cells.**
  - A whole-sheet area is 2^34 cells. `range_clear_contents` and the undo of an array formula now walk only the
    cells that exist, through the new `SheetData::places_in_block`. Clearing an absent cell had only stored an empty
    cell.
  - `set_user_array_formula` writes every cell of its area, so it refuses an area of more than 2^20 cells, the
    existing array limit. This is a DEVIATION, recorded in docs/excel-semantics/arrays.md.
  - The same cap applies to an array anchor's spill range in `check_cell`, so it holds for diffs and loads alike.
    A restored anchor claiming the whole sheet would make evaluation fill 2^34 cells. Refusing it at load is a
    DEVIATION, recorded in docs/excel-semantics/xlsx-storage.md.

The security review's two other findings are also fixed:
- `insert_sheet` refuses an id that is taken, which the undo of a sheet delete could carry;
- row heights and column widths refuse NaN and infinity.

**Atomicity.** A diff's validity depends on the diffs before it: the list can add the sheet a later diff edits, or
create the style that a later undo restores. So `apply_external_diffs` (now in `user_model/external_diffs.rs`) clones
the workbook, applies the list, and on any `Err` rebuilds the model from the clone with
`Model::from_workbook_with_clock`. A refused list therefore leaves the workbook unchanged, including the diffs that
applied before the refused one. The clone costs one pass over the workbook per call; a list that changes a cell already
evaluates the whole workbook. SPEC-0026 decision 4 has an "As built" note on this.

**Tests.** `test/user_model/test_external_diffs_untrusted.rs`:
- three real send queues with one index changed (style, sheet, formula);
- a list that builds on itself and still applies;
- 31 crafted diffs, reported by index, each refused with the workbook unchanged, after which an edit, an insert, an undo and a
  reload all work;
- whole-sheet areas;
- a taken sheet id and a NaN width.

**Follow-ups.**
- T-3423: a fuzz target for diffs, since the probe found more shapes than the ones listed.
- T-3426: bound the spill area many anchors claim in total; the per-anchor cap leaves n x 2^20.
