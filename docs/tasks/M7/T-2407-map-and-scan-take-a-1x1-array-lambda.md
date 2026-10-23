---
id: T-2407
title: "MAP and SCAN take a 1x1 array LAMBDA result, and SCAN a 1x1 array initial_value, as its one value (gen-m7)"
milestone: M7
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
MAP and SCAN use the one value of a 1x1 array LAMBDA result as the element (as BYROW and BYCOL do), and SCAN takes a 1x1 array
initial_value as its one value, as Excel gives; a multi-cell step stays the one `#CALC!` it is today.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 19 cells, 6 formulas, all `#VALUE!` in ours.
- 1x1 array steps: `MAP(..,LAMBDA(w,x,TRANSPOSE(w)))` over one-cell slices is each cell's value (b2078b5d702c691a A2162: 0.3, 2, 7, 0.1;
  f7e94a817e5385ac A842: FALSE, 0.5), `SCAN(..,Data!D5:D7,LAMBDA(w,acc,HSTACK(PROPER(acc))))` is "True", "0.3", "0" (7ea6ef6ceabacafd A87),
  `SCAN(..,LAMBDA(w,x,BYROW(w,_xleta.COUNT)))` is 0, 1, 1, 1 (2d9b48c6363e7a16 A2954: the accumulator stays a 1x1 array from step 2 on).
- 1x1 array initial_value: `SCAN(MINVERSE(Data!C7),..)` (MINVERSE of one cell is 1x1) is FALSE, `#NUM!` (23b5367c18059af2 A74), and
  `SCAN(LEN(WRAPROWS("Hello World",4,0.3)),Data!A7:D7,LAMBDA(z,y,z*y))` (WRAPROWS of a single value is 1x1, so LEN is {11}) is 22, 2.2E+16,
  6.6E+15, 0 (7ea6ef6ceabacafd A3615).
Ours: `MAP({1;2},LAMBDA(x,{5}))` and `SCAN({3},{1;2},LAMBDA(a,x,a+x))` are `#VALUE!` per element, while REDUCE with `{3}` gives 6 and BYROW
with `{5}` gives 5, 5 (checked 2026-10-22). Code: `fn_map` (functions/logical/map_reduce.rs), `fn_scan` (scan.rs) and
`Model::lambda_result_value`; reuse BYROW's 1x1 unwrap (`calc_result_to_array_node`, bycol_byrow.rs) rather than writing a second one.
Quirk entry: docs/excel-semantics/arrays.md "MAP and SCAN with a LAMBDA that returns a multi-cell range" (1x1 sentence added by this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 19 known-failure lines in eval/baseline/gen-m7.txt, 6 formulas.
- gen-m7/23b5367c18059af2.case.json A74 `=SCAN(MINVERSE(Data!C7),Data!C8:C9,LAMBDA(w,x,((w^Data!C4)>(0.3/Data!D6))))`: A74 Excel FALSE, ours #VALUE!; cells (2): A74, A75
- gen-m7/2d9b48c6363e7a16.case.json A2954 `=SCAN(REPLACE("a*",2,0,"1.5"),HSTACK(Data!Z1:Z2,{1;2}),LAMBDA(w,x,BYROW(w,_xleta.COUNT)))`: A2954 Excel 0.0, ours #VALUE!; cells (4): A2954:B2954, A2955:B2955
- gen-m7/7ea6ef6ceabacafd.case.json A87 `=SCAN(-((-2.5)/Data!B12),Data!D5:D7,LAMBDA(w,acc,HSTACK(PROPER(acc))))`: A87 Excel "True", ours #VALUE!; cells (3): A87, A88, A89
- gen-m7/7ea6ef6ceabacafd.case.json A3615 `=SCAN(LEN(WRAPROWS("Hello World",4,0.3)),Data!A7:D7,LAMBDA(z,y,z*y))`: A3615 Excel 22.0, ours #VALUE!; cells (4): A3615:D3615
- gen-m7/b2078b5d702c691a.case.json A2162 `=MAP(Data!B5:E6,{TRUE,FALSE;"TRUE",1},LAMBDA(w,x,TRANSPOSE(w)))`: A2162 Excel 0.3, ours #VALUE!; cells (4): A2162:B2162, A2163:B2163
- gen-m7/f7e94a817e5385ac.case.json A842 `=MAP(Data!C10:C11,Data!A5:B6,LAMBDA(acc,w,TRANSPOSE(w)))`: A842 Excel FALSE, ours #VALUE!; cells (2): A842, A843

## Acceptance criteria
- [ ] Visible parity: these 19 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] Unit tests: MAP and SCAN with a `{5}` step, SCAN with a `{3}` initial_value, and a multi-cell step that stays one `#CALC!`.
- [ ] The arrays.md entry's "Ours:" line names this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A multi-cell array initial_value (arrays.md: SCAN returning it is one `#CALC!`; REDUCE keeps it, T-2413's cell).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
