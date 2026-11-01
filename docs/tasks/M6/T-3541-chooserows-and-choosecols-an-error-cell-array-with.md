---
id: T-3541
title: "CHOOSEROWS and CHOOSECOLS: a referenced error cell is a 1x1 array, so an invalid index gives #VALUE! (gen-m6 arg-pairs cells)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CHOOSEROWS and CHOOSECOLS return Excel's #VALUE! when the array argument is a referenced #DIV/0! cell and the index is out of range or not numeric for a 1x1 array; a valid index (1.5, the cell itself) and a literal error keep returning #DIV/0! (see lookup.md).

## Context
Found by the T-3496 arg-pairs batch (excel-graph): Excel returns #VALUE!, the engine #DIV/0!. Related: T-2273 (error argument order).

## Acceptance criteria
- [ ] The cells below match Excel (probe the rule with the excel-probe skill first; record it in docs/excel-semantics/lookup.md).
- [ ] `make eval-baseline` shrinks gen-m6; `make check` green; reviewers approve.

## Out of scope
- Other functions and the unrelated cells of these cases.

## Visible-parity stage
Decision 0083: every cell below returns a wrong error; only some count in the in-scope visibly-off lock (it rose by 11 for the batch's 22 cells). The acceptance criteria are the stage.
Cells:
- 21db5363d711888d CHOOSECOLS(Data!G9, ...): A280 A286 A292 A304 A310 A316 A322 A334 A340
- ad47b47c4e97019d CHOOSEROWS(Data!G9, ...): A304 A310 A316 A328 A334 A340 A346 A358 A364
