---
id: T-3038
title: "Refuse a row or column insert that would push a table's cells off the sheet, as Excel does (InsertDeleteConflict)"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A whole-row (or whole-column) insert that would push any cell of a table off the sheet fails with an error and
leaves the sheet unchanged, even when the cells pushed off are empty. That is what Excel does.

## Context
- Evidence (excel-graph, 2026-10-28, T-2845; recorded in docs/excel-semantics/references.md "Structured references
  when the table changes"):
  - `Sales` on S!B1048573:C1048576 (header plus three data rows at the last sheet rows), insert above row 10:
    Graph `range('10:10')/insert` answers 409 `InsertDeleteConflict`, "This won't work because it would move cells
    in a table on your worksheet."
  - `Sales` on S!XFC3:XFD6, insert left of column C: the same 409.
  - The probe cases cannot be kept as eval cases (the oracle fill fails at the edit). The test is a unit test.
- Engine: `Model::insert_rows` / `insert_columns` in `crates/rustsheet-engine/src/actions.rs` refuse only when a
  *non-empty* cell would go past the last row (`dimension().max_row + row_count > LAST_ROW`). Today the insert runs and
  leaves the table where it is (the SPEC-0016 fallback, references.md). Add the table check next to the array-formula
  check (`can_insert_rows`), using the table's area (`tables.rs`), with an error message in the same style.
- Unprobed: whether a table that is merely moved to end exactly at the last row is accepted (an insert above a table
  ending at row 1048575). Treat it as allowed, since no table cell leaves the sheet, and say so in a comment.

## Acceptance criteria
- [ ] `insert_rows` and `insert_columns` return `Err` when a table's last row (column) would pass LAST_ROW
      (LAST_COLUMN), with unit tests for both and for the sheet left unchanged.
- [ ] references.md's T-2845 paragraph "Implemented in" is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other refusals of the same Graph error (array formulas already refuse; merged cells unprobed).

## Outcome
`insert_rows`/`insert_columns` refuse an insert that would push a table cell past the last row/column (`Model::refuse_table_off_sheet`, table_edits.rs); unit tests in `test_table_insert_limit.rs`. No eval delta.
