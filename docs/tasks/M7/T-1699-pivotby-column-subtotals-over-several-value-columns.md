---
id: T-1699
title: "PIVOTBY column subtotals over several value columns"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1324]
---

## Goal
PIVOTBY with column subtotals (col_total_depth ±2, or a column value sort over two fields with the depth omitted) and
two or more value columns gives Excel's cells instead of `#N/IMPL!`.

## Context
T-1324 implemented subtotals on both axes, but with several value columns Excel's column subtotal and Grand Total
columns are not the subtotals: most of their cells are `""` (like the Total columns with several values, P!Y97), yet a
few hold other columns' aggregates. probes/groupby-subtotals G!A781 (`PIVOTBY(A2:A11,B2:C11,D2:E11,SUM,,,,2)`):
the Apple and Pear subtotal columns and both Grand Total columns are `""` in every row; Banana (one child, 2025) has
its Sales-subtotal column showing Banana's Units (South 9, West 10, Total 19) and its Units-subtotal column showing every
row's Units total (East 9, North 6, South 9, West 23, blank 8, Total 55). G!A1141 (depth -2) shows the same two
columns; G!A1171 (Year over Product: 2025 has the one child Banana) shows only the first (2025's Units in the Sales
subtotal column), the Units subtotal column `""`. G!Z1201 (three values) is 8x28 (its spill was blocked: probe its
cells). Row subtotals over several values work (G!A751). docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY"
(T-1324 follow-up) records all of it. Code: `Model::pivotby_args` returns `#N/IMPL!` (`TODO(T-1699)`); slots and
cells are in pivotby.rs (`Slot::Subtotal`, `Model::grid_row`).

## Acceptance criteria
- [ ] A rule that explains G!A781, G!A1141 and G!A1171 (probe more shapes first: parents with one and with two
  children in other positions, three values, a header row), recorded in arrays.md; those cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
