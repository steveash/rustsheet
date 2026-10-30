---
id: T-2606
title: "Probe PIVOTBY's hidden Total gate with column subtotals, col_total_depth -1 and a LAMBDA returning empty text"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides which hidden PIVOTBY Total cells count in the three layouts T-2578 left unprobed, and `checked_totals` matches it.

## Context
From T-2578. arrays.md "PIVOTBY: an error column total over several value columns" gates each hidden Total cell of value column
`v` by the column line `(G + v) div V` (probes/t2578-pivotby-hidden-total-slot). Unprobed: (1) column subtotal lines
(`col_total_depth` 2 with two column fields; ours counts subtotal lines as lines, in grid order), (2) `col_total_depth` -1 (Total
columns first), (3) a LAMBDA that returns `""` in the gating line's cell (ours: a record there is enough). Probe with the decode
design of t2578-pivotby-hidden-total-slot: power-of-two values and `LAMBDA(v,IF(SUM(v)=k,NA(),SUM(v)))` per row total.
Code: `checked_totals`, `Grid::has_records` in functions/lookup_and_reference/pivotby.rs.

## Acceptance criteria
- [x] A probe case answers the three questions (excel-graph); the rule in arrays.md is updated with it.
- [x] Ours matches every new probe cell, or each difference is a baselined known failure with a task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/t2606-pivotby-total-gate-layouts` (excel-graph). `col_total_depth` -1 gates like 1 and a `""` gating cell still counts:
ours already matches (D1-D4, F1/G1). Column subtotal layouts do not follow the single-gating-line rule and the probe is confounded;
the 14 differing cells (of 28 subtotal cells) are baselined and filed as T-2967. arrays.md updated.
Question (1), column subtotals, is not answered: the result is recorded but undecoded and handed to T-2967; (2) and (3) are answered.
