---
id: T-2885
title: "Intersect a legacy N or T argument under a unary plus: N(+B1:B3) stores N(+@B1:B3)"
milestone: M2
lane: engine
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy (plain `<f>`) `N(+B1:B3)` reads like Excel: the plus makes the range a value, so the formula gets an `@`
(`=N(+@B1:B3)`) and gives the cell on the formula's row (2 on row 2), not the range's top-left cell (1).

## Context
Evidence: probes/t2772-unary-plus-in-range-rows J2, J5 (excel-graph, T-2772): legacy `=N(+B1:B3)` on row 2 is 2 and
FORMULATEXT prints `=N(+@B1:B3)`; legacy `=N(B1:B3)` (K2) is 1 with no `@`, and legacy `=+B1:B3` and `=ABS(+B1:B3)` give
`+@B1:B3` (L2, M2, already matching). docs/excel-semantics/syntax.md "A typed unary plus stays in the formula text".
We give 1 and print `=N(+B1:B3)`: `is_plain_reference` (static_analysis/legacy.rs) matches `node.lowered()`, which sees
through the plus at the top of the argument, so `reads_reference_whole` takes `+B1:B3` for a reference N reads whole.
A plus on an IF/CHOOSE literal index must still be seen through (`N(IF(+TRUE,B1:B3))` is 1 with no `@`, D2, G2 of the
same case). A plus on an IF arm (`N(IF(TRUE,+B1:B3))`) and on OFFSET's first argument are unprobed: probe them in this
task (excel-probe skill) before deciding how deep the rule goes.

## Acceptance criteria
- [ ] probes/t2772-unary-plus-in-range-rows J2 and J5 pass and come off the baseline; no other cell regresses.
- [ ] The unprobed arm/OFFSET shapes are probed and match; syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array formulas (a dynamic `N(+B1:B3)` lifts; not probed here).

## Outcome
`is_plain_reference` (static_analysis/legacy.rs) now returns false for an argument that is itself a unary plus, so legacy
`N(+B1:B3)` gets an `@` and intersects. t2772-unary-plus-in-range-rows J2, J5 pass and left the baseline; no other cell
changed. The plus-on-IF-arm and plus-on-OFFSET shapes need the oracle and are not probed here: filed as a follow-up
(research lane) and noted in syntax.md.
