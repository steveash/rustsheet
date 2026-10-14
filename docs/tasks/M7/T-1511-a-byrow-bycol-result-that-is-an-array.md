---
id: T-1511
title: "A BYROW/BYCOL result that is an array is #CALC!; ROWS(LAMBDA(x,x)) is 1; ISERROR of a LAMBDA is FALSE"
milestone: M7
lane: functions-logic
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A BYROW or BYCOL call whose LAMBDA (or function passed by name) returns an array is `#CALC!`, an uncalled LAMBDA is one value to ROWS, COLUMNS and the IS functions, and MAP over single values matches Excel.

## Context
Left over from T-1466 and T-1273 (docs/excel-semantics/arrays.md "Functions passed by name" and "Dynamic-array and LAMBDA function argument modes"):
- probes/m7-argument-modes F45:H45 `BYROW(A1:B3,LAMBDA(r,r))`: Excel `#CALC!`, ours a 3x1 array; probes/eta-reduced-names E49 `BYROW(A1:B3,_xleta.ABS)` and E101 `BYROW(A1:B3,_xleta.IF)`: Excel `#CALC!`, ours `#VALUE!` and `#ERROR!` (IF with one argument; what Excel's one-argument IF gives outside BYROW is unprobed); gen-m9/4df1b22d3ed63f58 Calc!A35 and gen-m9/95efb37a9874f82e Calc!A19: Excel `#CALC!`.
- probes/m7-argument-modes F44:H44 `ROWS(LAMBDA(x,x))` 1, `COLUMNS` 1, `INDEX(..,1,1)` `#CALC!`: ours `#VALUE!`.
- probes/eta-reduced-names E93 `ISERROR(_xleta.SUM)`: Excel FALSE, ours TRUE (probably the automatic `@` on a LAMBDA value; check `ISERROR(LAMBDA(x,x))` too, adding a probe question if needed).
- gen-m7/44ef4f5638e25bcd Calc!A7 `SUM(MAP(scalar,scalar,scalar,_xleta.MAX),..)`: Excel `#VALUE!`, ours a number (MAP over single values; probe MAP(1,LAMBDA(x,x)) and friends first).
Code: `fn_byrow`/`fn_bycol` and `calc_result_to_array_node` (bycol_byrow.rs), `call_lambda` (lambda.rs), ROWS/COLUMNS/INDEX on `CalcResult::Lambda`.

## Acceptance criteria
- [ ] probes/m7-argument-modes F44:H45 and probes/eta-reduced-names E49, E93, E101 pass, or the ones that cannot have their own task.
- [ ] gen-m9/4df1b22d3ed63f58 A35, gen-m9/95efb37a9874f82e A19 and gen-m7/44ef4f5638e25bcd A7 pass, or are explained in the Outcome.
- [ ] `make eval-baseline` records the fixes; no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
