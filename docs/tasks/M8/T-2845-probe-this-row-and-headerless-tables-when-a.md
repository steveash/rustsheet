---
id: T-2845
title: "Probe #This Row and headerless tables when a table moves or grows on a row insert"
milestone: M8
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/references.md says, with excel-graph evidence, what a `#This Row` reference and a
headerless table do when a whole-sheet row insert moves or grows the table.

## Context
T-2297 moves a table on a row insert at or above its header and grows it on one strictly inside its data,
keeping the text of every structured reference to it, except `#This Row`, which keeps SPEC-0016 step 1's
rule (a cell whose lowered area changes prints it; one off the data rows keeps its text). Unprobed:
- A `=Sales[@Qty]` (and `ROW(Sales[@Qty])`, `ISREF(...)`) on another sheet, on a row off the table's
  data rows before the edit and on them after it (`Sales` at S!B3:C7, insert above row 3 or row 6: F!A8):
  ours keeps the text and reads the table's row 8 (T-2297); before T-2297 it stayed `#VALUE!`. Also one on
  a data row (F!A5): ours prints the plain area it read (`=S!$C$6`).
- A calculated column `=[@Qty]*2` inside the table (S!D4:D7) under the same inserts: ours rewrites the
  moved cells to the plain area (`=$C5*2`); Excel very likely keeps `[@Qty]`.
- A table without a header row (`showHeaderRow` off, if Graph can make one): an insert at its first row.
  With a header, an insert at the first data row grows the table (T-2197); ours is a fallback (the
  reference prints the displaced area) until this is known.
- A table reaching the last sheet row (or column XFD) over empty cells, and a row (column) insert above
  (left of) it: ours runs the insert and leaves the table (a fallback); Excel may refuse it.
Use the excel-probe skill; cases go in eval/suites/probes/cases/table-edits with `insert_row` sheet ops.

## Acceptance criteria
- [x] Probe cases for the items above, oracled (excel-graph) and in the probes baseline.
- [x] references.md "Structured references when the table changes" records the findings; implementation
  tasks are filed for any mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 2026-10-28 on excel-graph. Four cases were added to probes/table-edits: table-insert-row-this-row-above-header,
-this-row-inside-data, -calc-column-above-header and -calc-column-inside-data. They add 39 new-coverage known
failures to eval/baseline/probes.txt and no regression. references.md "Structured references when the table
changes" records the findings:
- `#This Row` keeps its text through the move and the grow, in every cell (fixed cells on another sheet, moved
  cells beside the table, formulas inside it), and reads its own row in the new table. Ours rewrites every cell
  whose area changed to a plain area: T-3037.
- FORMULATEXT prints the `@` form, and `[@Qty]` inside the table. Ours prints the long form: added as evidence to
  T-1683.
- Excel refuses a row or column insert that would push a table off the sheet (Graph 409 InsertDeleteConflict, on a
  table at the last rows and one at the last columns). Ours runs the insert: T-3038. Those two probes cannot be kept
  as eval cases, because the oracle fill fails at the edit.
- A column of identical `Sales[@Qty]*2` formulas without calculatedColumnFormula metadata is not filled into the
  inserted row.
- The headerless-table item needs spec2xlsx support for a table with no header row first: T-3039.
