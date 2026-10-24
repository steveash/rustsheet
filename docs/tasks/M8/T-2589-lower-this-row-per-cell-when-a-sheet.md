---
id: T-2589
title: "Lower #This Row per cell when a sheet is duplicated or a table's sheet is deleted"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1699]
---

## Goal
A `#This Row` formula keeps its pre-SPEC-0016-step-2 value off the table's data rows when its sheet is duplicated
(`Model::duplicate_sheet`) or when the sheet of the table it names is deleted (`Model::take_tables_of_sheet`):
`#VALUE!`, as it was when `#This Row` was lowered per cell at parse time.

## Context
Follow-up of T-1699 (SPEC-0016 step 2). Both paths lower the sheet's shared formulas (R1C1, parsed at (1, 1)) with
`to_string_lowering(node, None, tables)`, so a `Node::ThisRowKind` has no row there and prints as the columns on the formula's own
row (`RC1`, `Node::this_row_relative`): right on every data row, but off them a duplicated sheet's formula reads the copy's cell
instead of `#VALUE!`, and a cross-sheet formula to a deleted sheet's table is `#REF!` instead of `#VALUE!`
(docs/excel-semantics/references.md "Structured references when the table changes", Implemented note). Excel copies the table
with the sheet (unprobed for the copy's formulas), so this is about keeping the engine's old values until T-2197's findings
decide the real rule. Fix: lower the cells whose formula holds a `#This Row` to one of those tables one by one, with their own
context (`to_string_lowering(node, Some(context), tables)` resolves the row), as `shift_cell_formula` re-enters a changed cell.

## Acceptance criteria
- [ ] test_this_row.rs: after `duplicate_sheet`, a `#This Row` formula on the copy off the source table's data rows is `#VALUE!`;
  after `take_sheet` of the table's sheet, a cross-sheet `#This Row` formula off the data rows is `#VALUE!`, on them `#REF!`.
- [ ] references.md's note on T-2589 is removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
