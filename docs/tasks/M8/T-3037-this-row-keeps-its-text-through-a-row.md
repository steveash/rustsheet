---
id: T-3037
title: "#This Row keeps its text through a row insert that moves or grows its table, and reads the formula's row in the new table (probes table-insert-row-this-row-*, -calc-column-*)"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A `#This Row` reference keeps its text through a whole-row insert that moves or grows its table, in every cell:
cells beside the table that move with the insert, cells on another sheet that stay put, and formulas inside the
table. Each one then evaluates at its own row against the table as it is after the edit, as Excel does.

## Context
- Evidence (excel-graph, 2026-10-28, T-2845): probes/table-edits table-insert-row-this-row-above-header,
  -this-row-inside-data, -calc-column-above-header and -calc-column-inside-data. These are 39 known-failure lines in
  eval/baseline/probes.txt. The rule is in docs/excel-semantics/references.md "Structured references when the table
  changes", T-2845 paragraph. Examples:
  - `Sales` on S!B3:C7 (Qty 1,2,4,8), F!A4:A8 `=Sales[@Qty]`, insert above row 3 (the table moves to B4:C8). Excel
    gives #VALUE!, 1, 2, 4, 8, because row 4 is now the header. Ours gives 1, 2, 4, 8, 8 (F!A4 to A7 keep reading their
    old row as plain areas, `=S!$C$8` and so on).
  - Insert above row 6 (the table grows): F!A6 is 0 (the new blank data row), A7 4 and A8 8. Ours: A6 4, A7 8.
  - S!E4:E7 `=Sales[@Qty]` beside the table move to E5:E8. Excel keeps the text and reads the same values. Ours rewrites
    them to `=$C$5` ... `=$C$8`, and the row-8 cell to `=#VALUE!`.
  - `=Sales[@Qty]*2` inside the table (S!D4:D7) keeps its text (it prints `=[@Qty]*2`). Ours rewrites it to a plain area.
- Engine: `table_edits.rs` (T-2297) keeps every other structured reference and applies SPEC-0016 step 1's fallback
  to `#This Row` ("a cell whose lowered area changes prints it"). Since T-1699 a `#This Row` is decided at the
  evaluating cell's row, so keeping the node as written is the whole change. A copy and paste already keeps it.
- FORMULATEXT's spelling (`Sales[@Qty]`, and `[@Qty]` inside its own table, where ours prints
  `Sales[[#This Row],[Qty]]`) is T-1683's. The text cells of these cases pass only once both tasks land.
- Cut and paste of a `#This Row` cell (desktop only) stays T-2325's.

## Acceptance criteria
- [ ] The value cells of the four cases above pass in `make eval` (F!A4:A8, C8, D8, H1:H6, and the calc-column B
      cells), and `make eval-baseline` removes their lines.
- [ ] Unit tests in `table_edits` cover a moved cell beside the table, a fixed cell on another sheet that goes from
      off the data rows to on them, and a formula inside the table.
- [ ] references.md's "Implemented in" for the T-2845 paragraph is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FORMULATEXT spelling (T-1683).
- Row deletes and column edits (unprobed for `#This Row`; file a probe task if needed).
