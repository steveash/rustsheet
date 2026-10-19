---
id: T-1806
title: "Probe the edge shapes of a cell-shaped call name"
milestone: M2
lane: engine
status: done
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
- [x] A probe case with oracle values covers each shape (value and FORMULATEXT) and passes.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe cases probes/cell-call-edges-a, -b, -c, -f and -g (excel-graph, 28 cells) cover every shape in the task that a
case file can express. An R1C1-shaped callee in an A1 workbook makes Excel refuse the workbook, so it has no case;
this is recorded in references.md. The column insert that pushes a called cell off the grid cannot be written in a
case file and is left unprobed.

Two fixes:
- In A1 text, an anchored or sheet-qualified cell directly followed by `(` is now a call of that cell
  (`reference_or_call_operand`), `#REF!` like Excel. It was a parse error, `#ERROR!`.
- `callee_operand` prints every cell callee bare, so FORMULATEXT gives `=$D$1(1)`, `=D$1(1)` and `=Sheet1!D1(1)`.

The other shapes already matched. New semantics entry in references.md, "Edge shapes of a cell-shaped call name",
and the parser test `test_cell_shaped_callee_is_call_of_cell` is extended.

Eval delta (probes): 5 new cases, 28 of 28 cells pass; regressions 0.
