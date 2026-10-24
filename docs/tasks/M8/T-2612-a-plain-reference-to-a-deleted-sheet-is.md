---
id: T-2612
title: "A plain reference to a deleted sheet is stored and exported as #REF!C4:C7, so a same-name sheet does not revive it (T-2340)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
After a sheet is deleted, a plain reference to it (`S!C4:C7`) loses the sheet for good, as in Excel: it stays
`#REF!` when a sheet with the old name is added later, and the xlsx export writes `#REF!C4:C7`.

## Context
T-2340 (references.md "Structured references under row deletes, column edits, convert, table and sheet
delete") matches Excel's FORMULATEXT `=SUM(#REF!C4:C7)` (probes/table-edits/table-delete-sheet) only in the
display (`stringify/wrong_reference.rs`): the stored R1C1 text keeps `S!`, so
- `add_sheet("S")` revives the reference (test_structured_references.rs
  `a_sheet_delete_is_a_ref_error_and_restore_brings_the_tables_back` pins the plain twin reading 0), and
- `to_excel_string` writes `S!C4:C7`, where Excel's file holds `#REF!C4:C7`.
The stored text keeps the name because `UserModel` undo of a sheet delete re-inserts the sheet and relies on
the other sheets' formulas reviving; a fix rewrites them on delete (as T-2340 does for structured
references, `Model::rewrite_table_references`) and records them for the undo (`TakenSheet`,
`Diff::DeleteSheet`). Probe first: does Excel revive nothing when a same-name sheet is added after a delete
(excel-graph `worksheets/add` after `DELETE worksheets`)? Also unprobed: a reference to a sheet that never
existed (a loaded `Nope!A1`), which the engine now displays as `#REF!A1`; and whether Excel accepts
`#REF!C4:C7` typed in (the lexer reads a cell-shaped tail after `#REF!` as part of it, so `#REF!LOG10(1)` lexes
as `#REF!` then `(1)`).

## Acceptance criteria
- [ ] A probe (excel-graph) of delete sheet S, then add sheet S: the plain `S!C4:C7` value and FORMULATEXT.
- [ ] The engine matches it; the export writes Excel's text; undo of the delete still restores the reference.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only undo (T-2325).
