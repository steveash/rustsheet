---
id: T-1947
title: "BYROW with _xleta.IF (IF called with one argument) is #CALC!, not #ERROR!"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`=BYROW(A1:B3,_xleta.IF)` gives one `#CALC!`, as Excel does (probes/eta-reduced-names E101; E102:E103 empty), instead of
three `#ERROR!` cells.

## Context
Split from T-1511. An eta-reduced IF called with one argument (the row reference) is an argument-count error in ours
(`#ERROR!`, `CalcResult::new_args_number_error`). Excel gives the whole BYROW `#CALC!`, which suggests that IF(row) is
accepted and lifts the 1x2 row into an array, which BYROW then refuses. What Excel's one-argument IF gives outside BYROW is
unprobed, and `=IF(TRUE)` may not be enterable. Probe `MAP({1,2},_xleta.IF)`, `BYROW({1;0},_xleta.IF)`,
`LET(f,_xleta.IF,f(TRUE))` and `LET(f,_xleta.IF,f(FALSE))` first, then match. Code: `eta_definition` (lambda.rs), `fn_if`,
`by_slices_lambda` (bycol_byrow.rs). docs/excel-semantics/arrays.md "Functions passed by name".

## Acceptance criteria
- [x] probes/eta-reduced-names E101:E103 leave eval/baseline/probes.txt; a probe of the one-argument eta IF is recorded in
  arrays.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/eta-if-one-argument (40 cells, excel-graph): an eta-reduced IF called with one argument is accepted. It
gives `#VALUE!` for any condition that is not an error, passes an error on, and lifts element by element over an array
or range condition, so BYROW and BYCOL over two-cell slices are one `#CALC!` and MAP is `#VALUE!` per element. Recorded
in arrays.md "Functions passed by name". `Model::if_pick` (functions/logical/mod.rs) now takes IF with only a condition
that way instead of giving an argument-count `#ERROR!`. Eval: probes/eta-reduced-names E101 fixed (E102:E103 were
its spill), the new case passes 40/40 (E69, `f()`, is left pending: excel-graph cannot decode its error), no regressions.
