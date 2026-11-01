---
id: T-3624
title: "SheetData::set_cell and set_row refuse columns outside the sheet, as check_layout does"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
No write stores a cell at a column outside the sheet (below 1 or past `LAST_COLUMN`), as none stores one
at a row outside it. The cells a `SheetData` holds are always ones `check_layout` accepts.

## Context
Found while reviewing T-3608: `SheetData::set_cell(3, i32::MIN, …)` stored a cell. `row_or_insert`
refuses rows outside `1..=LAST_ROW`, but nothing checked the column. `set_row` kept such columns too.
- `check_layout` already refuses a row whose columns leave `1..=LAST_COLUMN`, so a model in that state
  could not survive its own bytes.
- No reference can reach such a cell.
- Directive 5 treats workbook input and API callers as untrusted.
The owner asked for the fix (2026-11-01).

## Acceptance criteria
- [x] `set_cell` returns `None` for a column off the sheet before it creates a row; `set_row` drops such
      cells.
- [x] Tests: off-sheet columns at both ends, and `i32::MIN`/`i32::MAX`, store nothing and create no row.
      The four corners of the sheet do store. `check_layout` accepts what the writes leave. The tests
      catch the mutations that drop either check or let column 0 through.
- [x] The type doc says rows and columns outside the sheet hold no cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Callers that compute column numbers (structural edits): they reach `set_cell` and are bounded by it now.

## Outcome
- **Checks:** a one-line check in `set_cell`, and a filter in `set_row`.
- **File size:** `sheet_data.rs` is at exactly the 1,000-line limit after reflowing one narrow comment. The next
  change to it needs a split first, filed as T-3628: the `Row` type and its search.
- **Tests:** `test/test_sheet_data_bounds.rs`.
