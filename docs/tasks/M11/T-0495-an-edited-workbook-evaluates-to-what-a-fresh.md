---
id: T-0495
title: "An edited workbook evaluates to what a fresh load of it does (calipers pv26_014, pv23_019)"
milestone: M11
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A workbook evaluated after a series of edits holds the same values as the same workbook loaded
fresh, with the same edits applied, and evaluated once.

## Context
Found while verifying T-0453, and the same on main before it: in calipers
pv_goldens/pv26_014 (and pv26_2_014), loaded, evaluated, then edited 25 times (every
numeric constant at an even stride multiplied by 1.5 and incremented, one `evaluate()` per edit),
sheet 6 row 59 columns I, J, L read `OK` where the fresh load with the same edits reads `0.0`; in
pv23_019 (and pv23_027) sheet 3 row 97 columns G, H, I read `$ -` against `$ 0`. The first
divergence appears by the fifth edit. All four workbooks hold data tables and set
`calcPr iterate="1"`, so iterative calculation (`iteration.rs`), the data tables
(`data_tables.rs`) or a value carried over between evaluations (`FormulaValue` of a formula cell,
a stale spill) are the suspects. Reproduce with a release example that loads the golden, applies
the edits through `set_user_input` and compares `get_formatted_cell_value` of every cell with a
fresh load; find which evaluation first differs and which cell starts it.

## Acceptance criteria
- [ ] The divergence's cause is named in the Outcome, with the cell that starts it.
- [ ] A unit test reproduces it in a small workbook and passes; the four workbooks agree with a
      fresh load after the same 25 edits, or the task records which Excel behavior (evidence)
      makes the history matter, in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Data-table memoization (T-0453): the divergence is the same without it.
