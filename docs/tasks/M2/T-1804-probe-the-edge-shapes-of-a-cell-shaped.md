---
id: T-1804
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
T-1758 made `=Ab1(1)` a call of cell AB1 (`parse_named_call`, parser/operands.rs) but could not evidence these shapes: a column past XFD (`XFE1(1)`), a row 0 or past 1048576, `$`-anchored (`$A$1(1)`), sheet-qualified (`Sheet1!A1(1)`), R1C1-shaped in A1 mode (`R1C1(1)`), and non-ASCII letters (`ß1(1)`; ours keeps them unknown names). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers each shape (value and FORMULATEXT) and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
