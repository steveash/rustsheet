---
id: T-1271
title: "M7 array functions lift their scalar arguments; SORT reads sort_index and sort_order as lists"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: sonnet
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The scalar arguments of SEQUENCE, TAKE, DROP, EXPAND, MAKEARRAY, WRAPROWS, WRAPCOLS, TOCOL, UNIQUE and SORTBY lift over a range and an array (one call per element) as in Excel, and SORT reads an array sort_index as a list of sort keys with a sort_order of the same length.

## Context
SPEC-0011 rollout step 3 (docs/specs/SPEC-0011-scalar-argument-lifting.md). Lift the M7 scalars whose elements are values through `lift_call`, with their parameters in `Function::arg_kinds` (`Lift`, or `Whole` for the vector and list arguments). Lifted elements that are references (OFFSET, INDIRECT) or whole arrays (TAKE, DROP) are outside the spec. Design them here, on top of `lift_values` and its `element_node`, rather than as a new copy of the broadcast.

T-1083's probe probes/m7-argument-modes (excel-graph, 2026-10-09; docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"). Failing rows (F/G/H cells): 1-3 SEQUENCE (a range gives one row, an array `#N/IMPL!`), 4-5 TAKE, 6 DROP (a lifted TAKE or DROP element is the whole sub-array, column K), 7 EXPAND, 8 WRAPROWS, 9 TOCOL, 10-11 SORT (`SORT(A1:B3,{1;2})` is 3x2; `SORT(A1:B3,1,{1;-1})` is `#VALUE!`), 12 UNIQUE, 23 MAKEARRAY, 26 SORTBY, 42 WRAPCOLS pad_with. Reuse the existing lift helpers (T-1241 does the same for the M6 lookup functions; check whether it landed a shared helper first). The functions live in functions/lookup_and_reference and spill_functions.rs. Split per function family if it outgrows size M.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/6db767d720c5b55a Calc!A12 `=WRAPCOLS(Data!D3:D3,UNIQUE(Data!B3:E4,SWITCH(..)))` (wrap_count is UNIQUE's 2-D array): Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m7/a0cbc75358b429b6 Calc!A12 `=SORT(CONCAT(..),Data!D1:E3,Data!A9:C12,(..=..))` (range sort_index and sort_order): Excel `#N/A`, engine `#N/IMPL!`.
WRAPCOLS of a single value and wrap_count below 1 are T-1348; SORT's argument-error order is T-1351.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1782efcecd5d4b1b Calc!A19 `=TOCOL(Data!C8:D11,HSTACK(Data!E5:E6),..)` (ignore over a 2x1 array): Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m9/4df1b22d3ed63f58 Calc!A36 `=UNIQUE(Data!A6:D10,INDEX(Data!D4:D10,0))` (by_col over a 7-cell range): Excel `#SPILL!`, engine `#VALUE!`.

## Acceptance criteria
- [ ] The listed probes/m7-argument-modes cells and K1-K10 pass (H cells that need T-1242, INDEX of a single value, excepted); baseline updated.
- [ ] gen-m7/6db767d720c5b55a A12 and gen-m7/a0cbc75358b429b6 A12 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] gen-m9/1782efcecd5d4b1b A19 and 4df1b22d3ed63f58 A36 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
