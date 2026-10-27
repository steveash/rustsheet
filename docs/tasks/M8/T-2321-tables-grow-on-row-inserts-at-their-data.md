---
id: T-2321
title: "Tables grow on row inserts at their data edges and plain ranges onto them follow (T-2197 evidence)"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2297]
---

## Goal
A whole-sheet row insert at a table's data edges changes the table and the plain ranges onto it exactly as
Excel does on every cell of the probes/table-edits insert cases.

## Context
T-2197 probed Excel (excel-graph, `insert_row` sheet op = Graph `range('r:r')/insert` shift Down) on
`Sales` at S!B3:C7 (header 3, data 4-7), with and without a totals row 8; the findings are in
docs/excel-semantics/references.md "Structured references when the table changes". Two of them
contradict SPEC-0016's edge-case table, which T-2297 implements:
- Insert at `r == d1` (between header and first data row): Excel *grows* the table (header stays at 3,
  `ROWS(Sales)` 5); SPEC-0016 says fallback (table-insert-row-above-first-data).
- Insert at `r == t` (between the last data row and the totals row): Excel *grows* the table (data 5 rows,
  totals moves to 9); SPEC-0016 says fallback (table-insert-row-above-totals).
- A plain range onto the data column grows with those two inserts: `=SUM(S!C4:C7)` becomes
  `=SUM(S!C4:C8)` for the insert above row 4 (an ordinary insert at a range's first row would shift it to
  C5:C8) and for the insert above the totals row 8 (an ordinary insert just below a range leaves it).
  Without a totals row, an insert above row 8 changes neither the table nor C4:C7
  (table-insert-row-below).
Update SPEC-0016's edge-case table with these rows in the same PR. The eval adapter loads totals rows
since T-1671; table-insert-row-above-totals F!A12 (FORMULATEXT keeps `=Sales[[#Totals],[Qty]]`, ours
`=S!$C$9`) was unmasked by it and is this task's (decision 0128).

## Acceptance criteria
- [ ] Every cell of probes/table-edits/table-insert-row-* matches Excel (the T-1671 totals cells once it lands);
  their lines in eval/baseline/probes.txt disappear (`make eval-baseline`), no new known-failure lines.
- [ ] SPEC-0016's edge-case table rows for `r == d1` and `r == t` say grow, citing the probe cases.
- [ ] Unit tests in src/test/test_structured_references.rs cover both edges and the plain-range growth.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deletes, column edits, convert to range, table delete (T-2324 probes them).
