---
id: T-3241
title: "Print a qualified #This Row reference in Excel's @ form by the printing cell (FORMULATEXT [@Col] inside its table), keeping copy and fill outside the table working"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
FORMULATEXT and `get_cell_formula` print a `#This Row` structured reference written with its table name
(`T[[#This Row], [Col]]`, `T[@Col]`) in Excel's `@` form without the name (`[@Col]`, padded `[ @Col ]`) in a cell of
its own table, while copying, filling or moving such a formula outside the table still gives a formula that works there.

## Context
Split from T-3069. Evidence: probes/t2800-this-row-whitespace B3:E3 (`=[@Col]`, `=[ @Col ]`, excel-graph) and
probes/table-edits/table-insert-row-calc-column-inside-data F!A1:A2 (`=[@Qty]*2`); docs/excel-semantics/syntax.md
"Whitespace beside a missing argument". T-3069's first attempt set `StructuredReference::unqualified` at parse time when
the cell lies in the table (`Parser::containing_table_name`), which fixed those cells but was rejected in review
(PR #2415, Excel-fidelity round 2): the A1 print-then-parse paths (`Model::extend_to`, `extend_copied_value`,
`move_cell_value_to_area`, and anything feeding `get_cell_formula` text back to `set_user_input`) then print
`[@Col]` for a target outside the table, where the unqualified operand is refused, and the flag outlives a table
resize or delete. The `@` form must be decided by the printing cell, which `stringify` cannot do today (it has no
table geometry; `stringify/structured.rs::reference_text`). The same latent problem exists for an unqualified
`[@Col]` copied outside its table. Unprobed shapes (outside the table, header and totals rows) are T-3240.

## Acceptance criteria
- [x] probes/t2800-this-row-whitespace B3:E3 and table-insert-row-calc-column-inside-data F!A1:A2 pass; baseline updated.
- [x] Tests: an in-table `=T[@Col]` and `=[@Col]` copied and filled to a cell outside the table give a working
      formula; a table resize leaving the cell outside prints the qualified form.
- [x] `make check` green; reviewers approve.

## Out of scope
- The header/totals-row and outside-the-table spellings until T-3240 has evidence.

## Outcome
- The printing cell now decides the `@` form (`StructuredReference::text_at`, used by `stringify`
  whenever it prints at a cell). A `#This Row` reference written with the table name prints
  `[@Col]` (padding kept) on a data row of its own table; one written without the name prints with
  it (`T[[#This Row],[Col]]`, `T[Col]`) outside its table's range, so `extend_to`,
  `extend_copied_value`, a move or a resize give text that parses where it lands. Stored R1C1 text
  and the xlsx export are unchanged.
- The AST now holds parse-time table geometry: `StructuredReference::extent` (`TableExtent`) for
  `#This Row` references and unqualified ones. It relies on every change to the table list
  reparsing the formulas (`reset_tables`); an incremental reparse would have to refresh it.
  `Parser::table_named` is the one case-insensitive table lookup (it replaced two copies).
- Eval delta: probes known failures 2167 -> 2161 (fixed 6: t2800-this-row-whitespace S!B3:E3,
  table-edits/table-insert-row-calc-column-inside-data F!A1:A2), regressions 0. That case's
  A3, A4 and A6 still fail on the row insert lowering `#This Row` (T-3037).
- The outside-the-table spelling (`T[[#This Row],[Col]]` vs Excel's possible `T[@Col]`) and the
  header and totals rows stay unprobed (T-3240).
