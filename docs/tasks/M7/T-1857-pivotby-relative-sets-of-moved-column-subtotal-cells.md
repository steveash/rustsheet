---
id: T-1857
title: "PIVOTBY relative sets of moved column subtotal cells over several values"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1700]
---

## Goal
PERCENTOF and two-parameter LAMBDAs in a PIVOTBY column subtotal or Grand Total cell that Excel shows in
another column (one function over several value columns, T-1700) get Excel's relative set.

## Context
T-1700 found that one function over several value columns shows column subtotal and total cells in other
columns (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: column totals over several value columns";
`Placement`, `Shown::Moved` in functions/lookup_and_reference/pivotby.rs). Which relative set such a moved
cell's call gets is unprobed: the engine gives it the set of the subtotal or total whose cell it shows
(`RelativeSets::of_cell` with the source slot), not the set of the column it lands in.

## Acceptance criteria
- [ ] Probe (excel-probe skill) `LAMBDA(x,y,ARRAYTOTEXT(y))` and PERCENTOF over two value columns with
  col_total_depth 2 and relative_to 0-4, on data with a single-child parent and with the Grand Total shown in
  a parent's column (probes/pivotby-col-subtotals-values S!A1 data); record the rule in arrays.md.
- [ ] The probe cells pass; `make check` green; reviewers approve.

## Out of scope
- Where the cells land (T-1700).
