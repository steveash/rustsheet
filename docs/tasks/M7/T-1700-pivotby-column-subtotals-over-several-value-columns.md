---
id: T-1700
title: "PIVOTBY column subtotals over several value columns"
milestone: M7
lane: functions-lookup
status: done
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
(T-1324 follow-up) records all of it. Code: `Model::pivotby_args` returns `#N/IMPL!` (`TODO(T-1700)`); slots and
cells are in pivotby.rs (`Slot::Subtotal`, `Model::grid_row`).

## Acceptance criteria
- [x] A rule that explains G!A781, G!A1141 and G!A1171 (probe more shapes first: parents with one and with two
  children in other positions, three values, a header row), recorded in arrays.md; those cells pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Rule found (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: column totals over several value columns"): with
one function over w value columns, Excel lays the column lines out with the subtotals after their groups in the key
order before any value sort, then the total and each subtotal (shallowest first) write value column v into column
`f * w + k + v` (f its first group's line, k the lines it covers): it counts the covered lines, not their columns.
Writes into group columns are dropped, later writes win, and the subtotal and total columns then move with their lines
into the real layout (negative depth, value sort). The same rule explains the blank Total columns over several values
(P!Y97), and predicted the new case where it differs: one column group shows the last value's total in the first
Total column (probes/pivotby-total-columns-one-group). Paired functions and functions laid out as columns over one value
subtotal in place.
Hypotheses ruled out: "the parent at floor(parents / values) shows the Grand Total" (the earlier description) matches
every older probe but not -values2 S!A1 (eight single-child parents, two values: no total shown) or S!A85 (four
values: A, not B, shows it); the earlier brute force over index formulas on the leaf cells missed it because the
offset counts lines, not cells. The new rule was scored in a scratch simulator on all 13 older probes (1,276 cells, all
exact) before any code, then against three new excel-graph probe cases (32 shapes).
Code: `Placement` and `Shown` in pivotby.rs; the `#N/IMPL!` and the separate blank-Total branch are gone
(`Functions::blank_totals` became `one_over_several_values`).
Eval delta (probes): 2,073 known failures fixed (pivotby-col-subtotals-values 1,608, groupby-subtotals 465), none
regressed; known failures 4,446 -> 2,524. New cases pivotby-col-subtotals-values2, -values3 and
pivotby-total-columns-one-group pass except 151 cells of two other quirks found on the way: numbered "CUSTOM1"/"CUSTOM2"
labels for two LAMBDAs (130 cells, follow-up T-1854) and col_sort_order -3 with one column field over two values being
`#VALUE!` (21 cells, follow-up T-1855). The relative set of a moved cell is unprobed (follow-up T-1857).
