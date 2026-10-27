---
id: T-2846
title: "Probe which plain ranges grow with a table on a row insert at its first data row or totals row"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for which plain ranges grow with a table when a whole row is inserted at its first
data row or at its totals row, beyond the one probed range (the whole data column).

## Context
T-2197 (probes/table-edits, `Sales` on S!B3:C7, data rows 4-7) shows `=SUM(S!C4:C7)` becoming
`=SUM(S!C4:C8)` both for the insert above row 4 (table-insert-row-above-first-data) and above the
totals row 8 (table-insert-row-above-totals), where an ordinary insert would shift it or leave it.
T-2321 implements that quirk for every range inside the table's data block that touches the edge
(`Displacement::grown_corners`, stringify.rs): a hand generalisation. Unprobed shapes, each a formula
on sheet F for the same two inserts (and the plain insert above row 8 without totals as control):
`=SUM(S!C4:C5)` (touches the top edge only), `=SUM(S!C6:C7)` (touches the bottom edge only),
`=SUM(S!B4:C7)` (all data columns), `=SUM(S!C4:D7)` (sticks out right of the table),
`=SUM(S!C3:C7)` (header included), single cells `=S!C4` and `=S!C7`, a range on
the table sheet itself (S!E1 `=SUM(C4:C7)`), the absolute `=SUM(S!$C$4:$C$7)`, a defined name over
`S!$C$4:$C$7`, a conditional-format rule formula over the data, and a two-row insert at each edge.
references.md "Structured references when the table changes" records the finding.

## Acceptance criteria
- [ ] probes/table-edits cases for the shapes above, oracle-filled (excel-graph), with FORMULATEXT
      next to each formula.
- [ ] references.md "Structured references when the table changes" states which shapes grow; a
      follow-up engine task is filed for any shape where `grown_corners` disagrees.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (the follow-up task does).
