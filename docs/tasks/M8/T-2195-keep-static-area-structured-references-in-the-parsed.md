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
Every structured reference except `#This Row` keeps its written form in the engine's storage, and its area follows the table each time the table list changes.

## Context
SPEC-0016 rollout step 1. Today `=SUM(Sales[Qty])` is stored as `=SUM($A$2:$A$3)` (`structured_reference_operand`, parser/operands.rs). `Model::reset_tables` reparses every formula, so once the text is preserved, the area is re-derived from the current table. The xlsx export keeps today's lowered ranges until T-2196 writes table parts.

## Acceptance criteria
- [ ] `Node::StructuredReferenceKind { reference, lowered }` with a boxed `StructuredReference` (`table_name`, `specifier`, `table_reference`, `bare`), emitted for `ErrorKind` lowerings too. `Node::ungrouped()` steps into `lowered`; the `NODE_SIZE` assert stays green.
- [ ] Every `Node::ReferenceKind`/`Node::RangeKind` match site (`rg "Node::(ReferenceKind|RangeKind)"`) either calls `ungrouped()`, has an explicit arm delegating to `lowered` (`evaluate_node_with_reference`), or is shown not to receive parser output. A missed site falls through to an error value, never a panic.
- [ ] `to_rc_format`, `to_english_string`, `to_localized_string`, `to_string_displaced` and `to_string_moved` print the canonical long form with escapes; a bare table name prints as written. `to_excel_string` still prints the lowered area (a regression test pins it).
- [ ] Unit test: with `Sales` over A1:B3, `=SUM(Sales[Qty])` keeps its text (`get_cell_formula`). After `remove_table` the text is unchanged, and after `add_table` over A1:B4 it sums A2:A4.
- [ ] Every specifier and the column forms round-trip through `to_rc_format`. A property test over adversarial table and column names (`[`, `]`, `#`, `'`, `@`, spaces, empty, non-ASCII) checks `parse(print(x)) == x` and that printing never panics. A depth test covers many nested structured references.
- [ ] UPSTREAM.md notes the new `Node` variant.
- [ ] `#This Row` is unchanged (T-1699). `make eval` shows no regression; `make check` green; reviewers approve.

## Out of scope
- `#This Row` (T-1699); export of structured references and table parts (T-2196); the FORMULATEXT spelling (T-1683); tables that move or grow on row/column inserts (T-2197).
