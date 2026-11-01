---
id: T-3490
title: "Probe legacy formulas mixing a spill reference with a plain range, a bare spill reference and a scalar host"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3467]
---

## Goal
Know what Excel does with three legacy-formula shapes holding a spill reference, so the entry "A legacy formula with a
spill reference spills" in docs/excel-semantics/references.md covers them with evidence.

## Context
T-3467 probed only `=SIN(A1#)` stored as a legacy formula (probes/legacy-formula-spill-ref). It then applied the rule
to the whole formula, as it does for SWITCH. Three shapes are unprobed:
- a spill reference mixed with a plain range, `=SIN(A1#)+B1:B3`;
- a bare `=A1#`;
- a scalar host, `=SUM(A1#)+B1:B3`.

Place each, stored legacy (`legacy_formulas` in the case), on and off the spill's rows with a one-column spill.
Capture values, FORMULATEXT and `ISBLANK` of the cell below, as legacy-formula-spill-ref does. The engine code is
`removes_automatic_at` (static_analysis/legacy.rs) and `Model::promote_dynamic_formula_cells` (formula_cells.rs).

## Acceptance criteria
- [ ] A probe case with the three shapes is filled from excel-graph and recorded in references.md.
- [ ] Where the engine differs, it is fixed or a task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
