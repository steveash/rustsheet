---
id: T-2078
title: "LET, IF and CHOOSE hand an array of references on"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0014-reference-array-results.md
depends_on: [T-1956]
---

## Goal
LET, IF and CHOOSE in a reference position hand an array of references (a lifted OFFSET) on to the reader, as Excel does: `LET(x,OFFSET(A1,{0;1},0),INDEX(x,2,1))` is A2's value and `ROWS(LET(x,OFFSET(..),x))` 2.

## Context
Split from T-1956 (SPEC-0014 Rollout step 1), which built the `RefGrid` carrier, OFFSET as producer and the
INDEX, ROWS/COLUMNS and ISREF readers. Rule 7 of SPEC-0014 ("Pass-through"): probes/reference-lift-consumers E89
(`LET(x,r,INDEX(x,2,1))` 100), reference-lift-consumers-b1c E69-E81 (`SUM(IF(TRUE,r))`, `SUM(CHOOSE(1,r))`,
`SUM(LET(x,r,x))`, `ROWS(LET(x,OFFSET(..),x))`; the SUM forms also need T-2063's top-left walker). Today a LET binds
OFFSET's value view (`bind_argument` in functions/logical/let.rs goes through `eval_ref_or_index`), so the bound
grid is lost. Design: `Binding::Grid(RefGrid)` (an `Arc` clone, variable_stack.rs), read back by
`variable_reference`; IF and CHOOSE pick the arm in reference form (`if_impl` with `IfBranchMode::Aggregate`,
`choose_impl`) and return its `RefArg::Grid`.
T-2063 note: `SUM(IF(TRUE,r))` and `SUM(CHOOSE(1,r))` (b1c E69, E73) already pass through the aggregate walker's
reference reading. SUMPRODUCT reads a grid only from a bare OFFSET (`Model::evaluate_value_or_grid`, util.rs):
route its IF, CHOOSE and LET forms there too once they pass grids on (unprobed; Excel gives 0 by rule 7's reading).

## Acceptance criteria
- [x] The LET, IF and CHOOSE cells of rule 7 that need no aggregate walker pass (INDEX, ROWS/COLUMNS readers).
- [x] `VariableStack` charges newly built grids against `MAX_HELD_GRID_ELEMENTS = 4 * MAX_SIZE`, past it the
      binding is `#VALUE!`; recorded as a DEVIATION (decision 0014) under lookup.md, with a unit test.
- [x] OFFSET over a reference argument that is itself an array of references lifts again (SPEC-0014 rule 7:
      `ROWS(OFFSET(OFFSET(A1,{0;1},0),1,0))` 2), with a unit test of a deep nest staying bounded.
- [x] `make check` green; reviewers approve.

## Out of scope
- The aggregate walker (T-2063), per-element callers and VSTACK (T-2064), INDIRECT/TRIMRANGE (T-1957).

## Outcome
LET variables and LAMBDA parameters bind an array of references as `Binding::Grid` (an `Arc` clone when bound from
another variable), read back by `variable_reference`; `VariableStack` charges each distinct grid once against
`MAX_HELD_GRID_ELEMENTS = 4 * MAX_SIZE` (DEVIATION, lookup.md "Arrays of references held by LET variables are
bounded"). OFFSET whose reference argument is a grid (bare, through IF, CHOOSE, LET or a variable) lifts again over its
elements, broadcast with its own lifted arguments (`offset_over_grid`); SUMPRODUCT reads grids through IF, CHOOSE and
LET (`evaluate_value_or_grid`). New probes probes/reference-lift-pass-through (38 cells) and -b (10 cells, INDIRECT and TRIMRANGE grids; excel-graph) pin INDEX, ROWS,
ISREF, SUM, SUMPRODUCT and nested OFFSET through each pass-through; all pass. Eval delta: probes known failures
1698 -> 1694 (reference-lift-consumers E89, E145, E149; b1c E77), +48 passing cells in the new cases, no regressions.
