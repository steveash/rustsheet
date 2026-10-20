---
id: T-2078
title: "LET, IF and CHOOSE hand an array of references on"
milestone: M6
lane: functions-lookup
status: todo
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

## Acceptance criteria
- [ ] The LET, IF and CHOOSE cells of rule 7 that need no aggregate walker pass (INDEX, ROWS/COLUMNS readers).
- [ ] `VariableStack` charges newly built grids against `MAX_HELD_GRID_ELEMENTS = 4 * MAX_SIZE`, past it the
      binding is `#VALUE!`; recorded as a DEVIATION (decision 0014) under lookup.md, with a unit test.
- [ ] OFFSET over a reference argument that is itself an array of references lifts again (SPEC-0014 rule 7:
      `ROWS(OFFSET(OFFSET(A1,{0;1},0),1,0))` 2), with a unit test of a deep nest staying bounded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The aggregate walker (T-2063), per-element callers and VSTACK (T-2064), INDIRECT/TRIMRANGE (T-1957).
