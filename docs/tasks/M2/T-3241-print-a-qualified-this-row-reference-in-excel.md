---
id: T-3241
title: "Print a qualified #This Row reference in Excel's @ form by the printing cell (FORMULATEXT [@Col] inside its table), keeping copy and fill outside the table working"
milestone: M2
lane: engine
status: todo
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
- [ ] probes/t2800-this-row-whitespace B3:E3 and table-insert-row-calc-column-inside-data F!A1:A2 pass; baseline updated.
- [ ] Tests: an in-table `=T[@Col]` and `=[@Col]` copied and filled to a cell outside the table give a working
      formula; a table resize leaving the cell outside prints the qualified form.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The header/totals-row and outside-the-table spellings until T-3240 has evidence.
