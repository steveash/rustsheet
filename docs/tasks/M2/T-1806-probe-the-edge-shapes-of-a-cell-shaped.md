---
id: T-1806
title: "Probe the edge shapes of a cell-shaped call name"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior for the edge shapes of a cell-shaped call name is probed, recorded in docs/excel-semantics/references.md and matched.

## Context
T-1758 made `=Ab1(1)` a call of cell AB1 (`parse_named_call`, parser/operands.rs) but could not evidence these shapes: the last cell (`XFD1048576(1)`), a column past XFD (`XFE1(1)`), a row 0 or past 1048576, `$`-anchored (`$A$1(1)`), sheet-qualified (`Sheet1!A1(1)`), R1C1-shaped in A1 mode (`R1C1(1)`, `R[1]C[1](1)`), empty arguments (`=ab1()`), `=Ab1(1) B1`, and non-ASCII letters (`ß1(1)`; ours keeps them unknown names). Also a called cell pushed off the grid: ours turns `=XFD1(1)` after `insert_columns` into `=#REF!(1)` (`#ERROR!`); probe what Excel prints and evaluates. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers each shape (value and FORMULATEXT) and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
