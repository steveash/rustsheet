---
id: T-2195
title: "Keep static-area structured references in the parsed formula (SPEC-0016 step 1)"
milestone: M8
lane: engine
status: done
priority: P1
size: M
tier: opus
spec: docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md
depends_on: [T-2194]
---

## Goal
Every structured reference except `#This Row` keeps its written form in the engine's storage, and its area follows the table each time the table list changes.

## Context
SPEC-0016 rollout step 1. Today `=SUM(Sales[Qty])` is stored as `=SUM($A$2:$A$3)` (`structured_reference_operand`, parser/operands.rs). `Model::reset_tables` reparses every formula, so once the text is preserved, the area is re-derived from the current table. The xlsx export keeps today's lowered ranges until T-2196 writes table parts.

## Acceptance criteria
- [x] `Node::StructuredReferenceKind { reference, lowered }` with a boxed `StructuredReference` (`table_name`, `specifier`, `table_reference`, `bare`), emitted for `ErrorKind` lowerings too. `Node::ungrouped()` steps into `lowered`; the `NODE_SIZE` assert stays green.
- [x] Every `Node::ReferenceKind`/`Node::RangeKind` match site (`rg "Node::(ReferenceKind|RangeKind)"`) either calls `ungrouped()`, has an explicit arm delegating to `lowered` (`evaluate_node_with_reference`), or is shown not to receive parser output. A missed site falls through to an error value, never a panic.
- [x] `to_rc_format`, `to_english_string`, `to_localized_string`, `to_string_displaced` and `to_string_moved` print the canonical long form with escapes; a bare table name prints as written. `to_excel_string` and `to_excel_array_formula_string` still print the lowered area (a regression test pins both).
- [x] Unit test: with `Sales` over A1:B3, `=SUM(Sales[Qty])` keeps its text (`get_cell_formula`). After `remove_table` the text is unchanged, and after `add_table` over A1:B4 it sums A2:A4.
- [x] Every specifier and the column forms round-trip through `to_rc_format`. A property test over adversarial table and column names (`[`, `]`, `#`, `'`, `@`, spaces, non-ASCII; an empty column name prints without panicking, but `T[[#All],[]]` does not lex and Excel allows no empty header) checks `parse(print(x)) == x` and that printing never panics. A depth test covers many nested structured references.
- [x] Structural edits (split hint part a): every table is *fallback*, refined so a reference keeps its text when the edit leaves its area unchanged; otherwise `to_string_displaced`/`to_string_moved` print the displaced lowered area (today's text). Twin-formula tests pin today's values for row/column inserts, deletes and moves and for cut and paste, with the formulas on another sheet, plus one on the table's sheet. The move and grow/shrink classes, the fallback-name parameter and the per-edge-case-row tests moved to T-2297.
- [x] Sheet rename, delete and restore per SPEC-0016 "Sheet rename, delete and restore": rename updates the tables' `sheet_name` before `reset_parsed_structures`; delete prints the deleted sheet's tables as fallback in the other sheets' formulas and defined names (today's `#REF!`), removes those tables and keeps them with the taken sheet; `restore_sheet` puts them back. Tests pin today's values: same-sheet and cross-sheet formulas after a rename, a cross-sheet formula after a delete (and after re-adding a sheet with the old name), the tables after `restore_sheet`, and `to_excel_string` after a rename. Decide and test what `to_excel_string` writes for a `ParseErrorKind` holding a structured reference (after `remove_table`): never structured text without table parts.
- [x] UPSTREAM.md notes the new `Node` variant.
- [x] `#This Row` is unchanged (T-1699). `make eval` shows no regression; `make check` green; reviewers approve.

## Split hint
If the diff gets large, split it: (a) the node, transparency and printers, with structural edits on the fallback path for every table (today's behavior, so nothing regresses), then (b) the move and grow/shrink classes.

## Out of scope
- `#This Row` (T-1699); export of structured references and table parts (T-2196); the FORMULATEXT spelling (T-1683); tables that move or grow on row/column edits (T-2297; Excel's rules: T-2197).

## Outcome
Split per the split hint: this PR is part (a); the *move* and *grow/shrink* classes are T-2297.
- `Node::StructuredReferenceKind { reference, lowered }` (parser/structured_reference.rs) wraps every
  structured reference except `#This Row`, `ErrorKind` lowerings included; `ungrouped()` sees through it and
  a new `Node::lowered()` serves the sites that keep groups apart (legacy, ROW/COLUMN, OFFSET shape).
  Every `ReferenceKind`/`RangeKind` site was audited; exhaustive matches got arms.
- Storage and in-engine printers print the canonical long form with `'` escapes; the export
  (`to_excel_string`, `to_excel_array_formula_string`) prints the area. The R1C1 lexer now reads structured
  references (storage is R1C1).
- Structural edits and cut and paste: every table is *fallback*, refined so a reference keeps its text when
  the edit leaves its area unchanged; values equal today's (twin-formula tests for row/column inserts,
  deletes, moves and cut and paste).
- Sheet rename moves the tables with the sheet; delete lowers the other sheets' and defined names'
  references to the deleted sheet's tables (today's `#REF!`) and keeps the tables in `TakenSheet`;
  `restore_sheet` puts them back.
- After `remove_table` the formula is a parse error that keeps its text; the export writes it as `#REF!`.
- Eval delta: none expected (no gen-m8b case uses FORMULATEXT); see the PR.
