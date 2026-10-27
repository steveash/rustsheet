---
id: T-2843
title: "Probe #This Row and headerless tables when a table moves or grows on a row insert"
milestone: M8
lane: research
status: todo
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
Use the excel-probe skill; cases go in eval/suites/probes/cases/table-edits with `insert_row` sheet ops.

## Acceptance criteria
- [ ] Probe cases for the three items above, oracled (excel-graph) and in the probes baseline.
- [ ] references.md "Structured references when the table changes" records the findings; implementation
  tasks are filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
