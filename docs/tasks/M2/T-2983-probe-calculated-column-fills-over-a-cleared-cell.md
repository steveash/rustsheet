---
id: T-2983
title: "Probe calculated-column fills over a cleared cell and over identical stored formulas without calculatedColumnFormula"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
docs/excel-semantics/references.md ("Calculated columns: a formula typed into a table column") states, with
excel-graph evidence, two fill cases T-2958 had to guess: a column whose other data cells mix one formula and blanks,
and a column whose cells hold one identical stored formula with no saved `calculatedColumnFormula`.

## Context
Found by the T-2958 Excel-fidelity review. The engine (crates/rustsheet-engine/src/calculated_columns.rs,
`Model::calculated_column_targets`) fills a typed formula over the other data cells of a table column when they are
all blank or all hold one shared formula, and keeps no column-formula state on the table. Two choices in that rule
are unprobed:
1. Formulas mixed with blanks block the fill (unit test `no_fill_over_formulas_and_blanks_mixed`).
2. A column of identical stored formulas with no `calculatedColumnFormula` counts as a calculated column, so a
   different formula typed into one row re-fills it.

Probe cases (tag M2, under eval/suites/probes/cases/, modelled on probes/t2957-*: table `T` A1:C4, `Col` 1, 2, 4,
`Other` 10, 20, 40, FORMULATEXT of C2:C4 in F2:F4). Use A1 formulas (`=A2*2`), since an unqualified `[@Col]` does
not parse in the engine yet (T-2979):
- (a) stored `=A2*2`, `=A3*2`, `=A4*2` in C2:C4 (spec2xlsx writes no `calculatedColumnFormula`), then C3 entered
  `=A3*3`: does C2:C4 re-fill?
- (b) stored `=A2*2` in C2 and `=A4*2` in C4, C3 blank, then C4 entered `=A4*3`: does it fill C2 and C3?
- (c) as (b), with C3 entered `=A3*3` (the blank cell typed into): does it fill C2 and C4?

## Acceptance criteria
- [x] The probe cases are filled on excel-graph and in the probes baseline.
- [ ] references.md states both rules with the cells as evidence (first only; second is T-3025); an engine task is filed if ours differs. 
- [x] `make check` green; reviewers approve.

## Out of scope
- A stored `calculatedColumnFormula` on load (T-2969); the engine's unqualified structured references (T-2979).

## Outcome
Three probes filled on excel-graph (t2983-*). Rule: a column of stored identical formulas with no `calculatedColumnFormula`
is not a calculated column; typing beside it fills nothing (a: C5 typed beside stored C2:C4; c is a second witness).
The engine re-fills in (a), so it is a known failure in the probes baseline and engine task T-3024 is filed. (b) and (c)
pass in the engine only because the eval adapter types cells in row-major order with stored ones (also T-3024).
Whether a blank beside a real calculated-column formula blocks the fill stays unprobed (T-3025); a stored structured
formula variant of (a) is T-3023. The task's original (a) typed into C3, but an `entered` cell starts blank, so (a)
was redesigned (typed C5 of A1:C5). The acceptance criterion on the "cleared cell" half is not met; T-3025 carries it.
