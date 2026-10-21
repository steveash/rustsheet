---
id: T-1855
title: "PIVOTBY col_sort_order past the first value column with one column field"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
PIVOTBY's col_sort_order accepts and rejects the same value-column indices as Excel.

## Context
Found by T-1700. `PIVOTBY(Data!A2:A9,Data!J2:J9,Data!E2:F9,LAMBDA(x,ARRAYTOTEXT(x)),,,,1,-3)` (one column
field, two value columns, so -3 names the second value column) is `#VALUE!` in Excel
(probes/pivotby-total-columns-one-group S!A73, N!A73 "ERR"); the engine sorts and spills 4x5. With two
column fields, -3 (the first value column) sorts (probes/pivotby-col-subtotals-values S!A111). Not known:
whether a column value sort may only name the first value column, or whether the bound counts something
else (row fields?). GROUPBY's and the row sort's rules: docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY"
(sort_order past the keys). Code: `Model::sort_order` (groupby.rs), called by `Model::pivotby_args`.

## Acceptance criteria
- [x] Probe col_sort_order 2, 3, -3, 4 with one and two column fields and one, two and three value columns
  (and the row sort for contrast); record the rule in arrays.md.
- [x] S!A73 and the new probe cells pass; `make check` green; reviewers approve.

## Out of scope
- Sort semantics once the index is valid.

## Outcome
Probed col_sort_order and row_sort_order 2, ±3, ±4, 5 over one and two row and column fields and one to
three value columns (probes/pivotby-sort-order-past-keys, 167 excel-graph cells). Excel's col_sort_order
counts one value column past the column fields (the first), whatever the value columns or functions;
row_sort_order counts every output value column. `Model::pivotby_args` now bounds the column sort by one
value column; recorded in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY". Eval: probes known failures
1600 -> 1579 (21 fixed, incl. pivotby-total-columns-one-group S!A73/N!A73), new case 167/167, 0 regressions.
