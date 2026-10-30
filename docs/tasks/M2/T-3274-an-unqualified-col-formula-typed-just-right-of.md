---
id: T-3274
title: "An unqualified [@Col] formula typed just right of a table resolves against the table it grows, fills and prints [@Col] (rule 7f, t3003-unqualified-structured-d2)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`=[@Col]*2` typed into the cell just right of a table's data row is accepted, as Excel accepts it. The table grows by
the column (header `Column1`), every data row of the new column holds `=[@Col]*2` and computes against its own row,
and FORMULATEXT of each prints `=[@Col]*2`. Today the engine refuses the entry and stores `#ERROR!`.

## Context
Evidence: docs/excel-semantics/references.md "Calculated columns" rule 7(f) (T-3003, excel-graph 2026-10-30). Table
`T` A1:C4 (`Col` 1, 2, 4), `=[@Col]*2` typed into D2. `T` grows to A1:D4, D1 is the text `Column1`, D2:D4 are 2, 4,
8, and FORMULATEXT(D2:D4) is `=[@Col]*2`. Excel resolves the unqualified reference against the table the new column
joins.

Case owned (decision 0096; every failing cell, split from T-3218 at planning 2026-10-30):
- probes/t3003-unqualified-structured-d2 S!D2, S!D3, S!D4, S!F2 (decision 0186), S!F3, S!F4, S!H1, S!H4, S!J1, S!J2, S!J3, S!J4, S!K1
  (13 visibly-off cells; D2:D4 are blank, since T-3213 refuses the typed entry, and `COLUMNS(T[#All])` is 3).
  S!K2 (`=TYPE(D2)`, 1) passes by coincidence since T-3213: a blank D2 also gives 1.

Why it fails (crates/rustsheet-engine/src/calculated_columns.rs, `Model::table_expansion`):
- `table_expansion` grows the table only when `parses_as_typed` accepts the formula in its cell.
- T-2979 (done) reads an operand starting with `[` as a reference to the table containing the cell
  (`Parser::unqualified_structured_reference` and `containing_table_name` in
  crates/rustsheet-engine/src/expressions/parser/structured_reference.rs), and refuses it outside every table.
- D2 is outside `T` until the growth, so the parse is refused, no growth happens, and the cell stores the parse error.

The A1 form of the same entry (`=A2*2`, t2957-expand-right-data) already grows and fills. So only the parse context
of the grown cell is missing; the growth and fill paths exist. The fix decides, in one place, that a cell
`table_expansion` would add to a table parses as inside it. Options include a provisional parse against the grown
table, or `containing_table_name` taking the expansion into account. Either way, a refused entry must leave the
table unchanged (`restore_table`) and the facade's undo must still restore it.

Neighbouring tasks:
- T-3218 edits the guards at the top of `table_expansion` for constants and a totals row; this task touches only
  the parse check. Whichever lands second rebases.
- T-3213 changes what a typed formula that does not parse leaves behind. After this task, this entry parses.

## Acceptance criteria
- [ ] All 13 listed cells of probes/t3003-unqualified-structured-d2 pass bit-exact. Its F lines leave
      eval/baseline/probes.txt via `make eval-baseline`, with 0 regressions in every suite.
- [ ] Every passing t2957-* and t3003-* case keeps passing. That includes expand-right-data/-header, occupied-*
      (rule 7d: an occupied neighbour still stops the growth) and header-formula-*.
- [ ] Engine tests:
      - `=[@Col]*2` typed beside a table grows it and fills the column.
      - The stored formula carries the `[T]` prefix (`StructuredReference::stored_text`), so it re-parses
        unqualified after a save/load round trip.
      - `[@Other]` (another column of the grown table) resolves too.
      - Two unprobed shapes keep today's behavior, with a comment in each test saying it is unprobed: with an
        occupied neighbour (rule 7d, no growth) `=[@Col]*2` is refused as it is outside any table, and an unknown
        column `[@Nope]` beside the table is refused and the table does not grow.
- [ ] Facade test: undo of the edit restores the table and clears D2:D4.
- [ ] references.md rule 7(f) drops "The engine refuses this entry as a parse error (T-2979)" and states what we
      do. `make check` green; reviewers approve.

## Out of scope
- `=[@Col]*2` typed into the header cell D1, or just below the table: both are unprobed. File a research probe task
  if needed.
- Unqualified references typed beside a table that another table overlaps (unprobed).
- Constants and a totals row (T-3218), and row growth (T-3273).
