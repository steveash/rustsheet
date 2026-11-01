---
id: T-3320
title: "Probe typing below a table: calculated-column fill into the new row, totals row, occupied row, two rows down"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The unprobed neighbours of references.md "Calculated columns" rule 7(e) (typing just below a table grows it by a
row) rest on oracle evidence, and the engine matches it or a follow-up task says what to change.

## Context
T-3273 implemented rule 7(e) from probes/t3003-below-last-row and -const, which type into A5 below a column of
constants and read back only ROWS/COLUMNS and A5. It pinned these as unprobed, in
crates/rustsheet-engine/src/test/test_calculated_columns.rs (`unprobed_neighbours_below_the_table_do_not_grow_it`,
`a_formula_typed_below_the_table_fills_no_calculated_column`, `a_table_on_the_last_row_does_not_grow_down`):
- a formula typed into C5 below an empty column, or below a calculated column (`=[@Col]*2` in C2:C4): the engine grows
  the table and fills nothing, neither C5's column into C2:C4 nor the calculated formula into C5's neighbours;
- below a table with a totals row (typing into A6 under totals row 5): no growth;
- a new row whose other cells are not blank (B5 holds 1, type into A5): no growth;
- two rows below (A6), outside the columns (D5 or E5), or into a row another table reaches: no growth.
Write one probe case per question with the excel-probe skill (table `T` A1:C4 as in probes/t3003-*, readouts
`ROWS(T[#All])`, `COLUMNS(T[#All])`, FORMULATEXT/ISFORMULA of the filled cells), fill it through excel-graph and
record the answers in references.md rule 7(e).

## Acceptance criteria
- [ ] A probe case per question above is committed with excel-graph values.
- [ ] references.md rule 7(e) records each answer; a task is filed for every answer the engine does not match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (file a task per mismatch instead).
