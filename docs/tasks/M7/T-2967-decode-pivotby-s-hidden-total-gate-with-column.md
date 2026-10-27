---
id: T-2967
title: "Decode PIVOTBY's hidden Total gate with column subtotal lines (T-2606 confounds)"
milestone: M7
lane: functions-lookup
status: todo
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
- [ ] Excel evidence decides which hidden Total cells count with subtotal lines; ours matches every cell of the probe and its t2606 baseline entries are removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
