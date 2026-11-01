---
id: T-2967
title: "Decode PIVOTBY's hidden Total gate with column subtotal lines (T-2606 confounds)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`checked_totals` matches Excel's hidden Total gate for PIVOTBY layouts with column subtotal lines (`col_total_depth` 2 / -2).

## Context
T-2606 probed these (probes/t2606-pivotby-total-gate-layouts P!A1-B14); the result is not monotone in a row's records and is
confounded where the row total equals a subtotal cell (arrays.md, "PIVOTBY: an error column total over several value columns is #VALUE! (T-2311)").
Failing cells (14 of 28): P!A1, A2, A4, A6, A7, A9, A11, A13, B2, B4, B6, B7, B11, B13. Redesign the probe so no row total equals a subtotal or group cell (every row two records
in the same outer group or none), decode all layouts as T-2578 did, then fix `checked_totals` in
functions/lookup_and_reference/pivotby.rs.

## Acceptance criteria
- [x] Excel evidence decides which hidden Total cells count with subtotal lines; ours matches every cell of the probe and its t2606 baseline entries are removed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Decoded with a new probe, probes/t2967-pivotby-subtotal-gate (359 cells, excel-graph, 2026-10-28): every row subset of
five layouts (p/x,p/y,q/x,q/y at col_total_depth 2 and -2; p/x,p/y,p/z,q/x; p/x,p/y,q/x with 3 value columns; the first
sorted by value). Each row r is paired with a filler row holding every leaf, and a TEXTJOIN trigger names exactly one cell's
records, so no row total is mistaken for a subtotal. The hidden Total gate is the T-1700 placement rule: the total and each
subtotal write value v into placement column f*V + k + v. A write that lands on a group column is dropped but still computed,
in the rows with a record in that group, and an error there is `#VALUE!`. A write that lands on a subtotal or Total column is
shown and never counts. Without subtotals this reduces to T-2578's `(G + v) div V` gate.
`Placement::checked_totals` (new module pivot_placement.rs, split out of pivotby.rs for the file-size gate) takes the
dropped writes of `written_cells`, and `check_totals` aggregates a subtotal slot over its own records.
Eval: probes known failures 2156 -> 2142 (t2606's 14 cells fixed), plus 359 new passing cells, 0 regressions. The old
gate passes 168 of the new 359. arrays.md updated.
