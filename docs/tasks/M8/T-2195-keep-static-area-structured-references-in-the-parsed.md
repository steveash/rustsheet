---
id: T-2195
title: "Keep static-area structured references in the parsed formula (SPEC-0016 step 1)"
milestone: M8
lane: engine
status: todo
priority: P1
size: M
tier: opus
spec: docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md
depends_on: [T-2194]
---

## Goal
Every structured reference except `#This Row` keeps its written form in storage and export, and its area follows the table each time the table list changes.

## Context
SPEC-0016 rollout step 1. Today `=SUM(Sales[Qty])` is stored and exported as `=SUM($A$2:$A$3)` (`structured_reference_operand`, parser/operands.rs). `Model::reset_tables` reparses every formula, so once the text is preserved, the area is re-derived from the current table.

## Acceptance criteria
- [ ] `Node::StructuredReferenceKind { reference, lowered }` with a boxed `StructuredReference`. `Node::ungrouped()` steps into `lowered`. Each of the `Node::ReferenceKind` match sites either calls `ungrouped()` or is shown not to receive parser output.
- [ ] Every printer prints the canonical long form, with escapes: `to_rc_format`, `to_english_string`, `to_localized_string`, `to_excel_string`, `to_string_displaced`, `to_string_moved`. A bare table name prints as written.
- [ ] Unit test: with `Sales` over A1:B3, `=SUM(Sales[Qty])` keeps its text (`get_cell_formula`). After `remove_table` + `add_table` over A1:B4, it sums A2:A4.
- [ ] Every specifier and the column forms round-trip through `to_rc_format` and `to_excel_string`. An xlsx round trip keeps the structured text.
- [ ] `#This Row` is unchanged (T-1699). `make eval` shows no regression; `make check` green; reviewers approve.

## Out of scope
- `#This Row` (T-1699); the FORMULATEXT spelling (T-1683); tables that move or grow on row/column inserts.
