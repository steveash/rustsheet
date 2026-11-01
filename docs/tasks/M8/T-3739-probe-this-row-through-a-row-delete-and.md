---
id: T-3739
title: "Probe #This Row through a row delete and a column edit its table follows (T-3037 implemented row inserts only)"
milestone: M8
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for `#This Row` references through a whole-row delete and a whole-column insert or delete that
their table follows (it moves, shrinks or grows), so the engine's rule for them is evidenced, not hand.

## Context
- T-3037 made a `#This Row` reference to a table that follows a row insert keep its text and read its cell's row
  in the edited table, on another sheet, beside the table and inside it (T-2845 table-insert-row-this-row-*,
  -calc-column-*). Row deletes and column edits the table follows (a delete above it, or inside its data rows leaving
  one; a column insert or delete left of or inside it) keep the old rule: a moved cell's `#This Row` is rewritten to
  the plain area it read, and an unmoved cell's where its area changes. See references.md "Structured references
  when the table changes", the T-2845 bullet and the "Implemented in" paragraph.
- Engine: `Model::row_edit` (it gives deletes no kept tables) and `Model::moved_text` (table_edits.rs,
  formula_cells.rs), the `Rewrite::Displace` kept-table branch in stringify/structured.rs (row inserts only).
- Use the layout of probes/table-edits/table-insert-row-this-row-above-header (Sales on S!B3:C7, F!A2:A8
  `=Sales[@Qty]`, S!E4:E7 beside it, a calculated column) with: delete row 2 (moves up), delete row 5 (shrinks; the
  F!A cell on the deleted row and the ones below), insert a column left of the table and inside it, delete a column
  left of it.

## Acceptance criteria
- [ ] Probe cases under eval/suites/probes/cases/table-edits with excel-graph values (excel-probe skill).
- [ ] Where Excel differs from the engine, a follow-up engine task is filed; the finding is recorded in
      references.md in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cut and paste (desktop only, T-2325); FORMULATEXT spelling (T-1683).
