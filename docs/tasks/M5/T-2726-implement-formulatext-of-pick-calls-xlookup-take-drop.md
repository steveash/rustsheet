---
id: T-2726
title: "Implement FORMULATEXT of pick calls, XLOOKUP/TAKE/DROP/LET/SWITCH/IFS results and array INDIRECT (probes/formulatext-pick-scalars-and-arrays)"
milestone: M5
lane: functions-lookup
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The engine's FORMULATEXT matches Excel on every cell of probes/formulatext-pick-scalars-and-arrays.

## Context
T-2529 probed these forms; the engine misses 43 of 49 cells: 26 `#ERROR!` (scalar forms, XLOOKUP/TAKE/DROP spill anchors), 14 `#VALUE!` (each element of the array IF/CHOOSE/INDIRECT forms, C1:T1; Calc!O1 `INDIRECT({"Data!A2"})` joined them when T-2577 made a 1x1-array INDIRECT a one-element array of references, which FORMULATEXT must read per element) and 3 empty spill cells (V1, X2, Z2). Findings are in docs/excel-semantics/references.md ("FORMULATEXT of the forms T-2526 left open"). Code path: `fn_formulatext`.

## Acceptance criteria
- [x] All cells of the probe pass and its lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_formulatext` reads a pick call (IF, CHOOSE, SWITCH, IFS, LET, INDEX, OFFSET, INDIRECT,
XLOOKUP, TAKE, DROP, TRIMRANGE) as SUBTOTAL does, through `evaluate_picked_reference_arg`: a picked
cell or range is read as before, a lifted INDIRECT element by element (`formula_texts_of_grid`), a
failed pick is its own error and a value is `#VALUE!` per element with error elements kept. All 49
cells of probes/formulatext-pick-scalars-and-arrays pass (was 6), and the other FORMULATEXT probes
still pass. docs/excel-semantics/references.md updated.
