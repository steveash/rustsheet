---
id: T-1511
title: "A BYROW/BYCOL result that is an array is #CALC!; ROWS(LAMBDA(x,x)) is 1; ISERROR of a LAMBDA is FALSE"
milestone: M7
lane: functions-logic
status: done
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
- Unprobed (T-1466 fidelity review): BYCOL/BYROW with `1/0` or `{1,2}` as the function, `MAKEARRAY(2,2,_xleta.ABS)` and `BYROW(A1:B3,_xleta.NOW)` (excel-graph cannot decode them; try `ERROR.TYPE(..)`), special forms passed by name (`_xleta.LET`, `_xleta.LAMBDA`, `_xleta.SUBTOTAL`, `_xleta.OFFSET`) and `LET(f,_xleta.SUM,f())`: probe them and match.
Code: `fn_byrow`/`fn_bycol` and `calc_result_to_array_node` (bycol_byrow.rs), `call_lambda` (lambda.rs), ROWS/COLUMNS/INDEX on `CalcResult::Lambda`.

## Update (T-1664)
BYROW/BYCOL over a range now give one `#CALC!` for a multi-value result: m7-argument-modes F45:H45, eta-reduced-names E49 and gen-m9 4df1b22d3ed63f58 A35 pass. Still open here: E101-E103 (one-argument `_xleta.IF` gives `#ERROR!`, never an array), E93, F44:H44, gen-m9/95efb37a9874f82e A19 (`#SPILL!`, probably FISHERINV/RADIANS not lifting over a range) and gen-m7/44ef4f5638e25bcd A7.

## Acceptance criteria
- [x] probes/m7-argument-modes F44:H45 and probes/eta-reduced-names E49, E93, E101 pass, or the ones that cannot have their own task.
- [x] gen-m9/4df1b22d3ed63f58 A35, gen-m9/95efb37a9874f82e A19 and gen-m7/44ef4f5638e25bcd A7 pass, or are explained in the Outcome.
- [x] `make eval-baseline` records the fixes; no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/lambda-value-arguments (excel-graph, 2026-10-19; 41 cells) probes an uncalled LAMBDA or function name as a value, and
MAP over single values. The engine now matches all 41 cells:
- ROWS and COLUMNS of an uncalled LAMBDA are 1 (`value_extent`).
- INDEX gives the LAMBDA back for row and column 0 or 1 (`index_select`), and a cell shows it as `#CALC!`.
- The IS* tests treat it as a non-error, non-text value (`is_predicate`); ISEVEN and ISODD give `#VALUE!` (`parity_of`).
- MAP refuses a single value as its second or later array (`by_slices_source`'s `refuse_scalar`).
arrays.md has two new entries, "An uncalled LAMBDA is one value" and "MAP over single values".
Fixed: m7-argument-modes F44:H44, eta-reduced-names E93 and gen-m7/44ef4f5638e25bcd A7. gen-m9/95efb37a9874f82e A19 and the
T-1664 cells already passed on main.
Left: eta-reduced-names E101:E103 (`_xleta.IF` with one argument), split to T-1944 since it needs a probe of the one-argument
IF. T-1946 probes the unprobed edge cases that review found: MAP with a blank or error later array, and INDEX of a LAMBDA out of range.
Eval delta: probes known failures 2133 -> 2129 (4 fixed, 1 new case passing 41/41); gen-m7 A7 fixed; REGRESSIONS 0.
