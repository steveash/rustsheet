---
id: T-2309
title: "MAP and SCAN: a LAMBDA step that returns a multi-cell array makes the whole call #CALC! (gen-m7)"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
A MAP or SCAN whose LAMBDA returns a multi-cell array or range for an element (HSTACK, FILTER, TRIMRANGE, a BYROW column, an array accumulator) is a single #CALC!, as Excel gives, not a spilled grid of #VALUE!.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "MAP and SCAN with a LAMBDA that returns a multi-cell range".

Ours turns each non-scalar step into `#VALUE!` in its slot (`lambda_result_value`) and spills the grid; Excel's result is one `#CALC!` cell with nothing spilled: `MAP(Data!C4:D12,Data!A8:B12,LAMBDA(x,acc,HSTACK(acc,Data!A7:D11,acc)))` (611ce3bbfab27cef A944: Excel `#CALC!` and empty B944..., ours 18 `#VALUE!` cells), `SCAN(Data!B8,Data!B6:E11,LAMBDA(acc,v,HSTACK(..)))` (b24ce74f08d3f91a A430), `SCAN(..,LAMBDA(y,acc,TRIMRANGE(Data!B11:D12)))` (2c5ea078f25443a2 A1717), `SCAN(E6,<scalar>,LAMBDA(y,acc,FILTER(..)))` (7307132bb8820f17 A619), `MAP(Data!$D4,LAMBDA(v,(D4*v)^BYROW(B3:C11,..)))` (7307132bb8820f17 A1324) and a SCAN whose initial value is an array and whose LAMBDA returns the accumulator (9359f03972e94026 A1705). arrays.md already records that a BYROW LAMBDA returning a row is `#CALC!`, and T-1792 made a multi-cell range result `#CALC!` for MAP and SCAN (probes/map-scan-range-result, which must keep passing); an array value (HSTACK, FILTER, a BYROW column, an array accumulator) and TRIMRANGE's result still take the per-element `#VALUE!` path. Code: `lambda_result_value` and its callers in functions/logical/ (scan.rs, map_reduce.rs).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 63 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 63 known-failure lines in eval/baseline/gen-m7.txt, 6 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/2c5ea078f25443a2.case.json A1717 `=SCAN((IFERROR(Data!A5,Data!$E6)*(2+Data!$D6)),Data!A9:B10,LAMBDA(y,acc,TRIMRANGE(Data!B11:D12)))`: A1717 Excel #CALC!, ours #VALUE!; cells (1): A1717
- gen-m7/611ce3bbfab27cef.case.json A944 `=MAP(Data!C4:D12,Data!A8:B12,LAMBDA(x,acc,HSTACK(acc,Data!A7:D11,acc)))`: A944 Excel #CALC!, ours #VALUE!; cells (18): A944:B944, A945:B945, A946:B946, A947:B947, A948:B948, A949:B949, A950:B950, A951:B951, A952:B952
- gen-m7/7307132bb8820f17.case.json A619 `=SCAN(Data!E6,(ROUND(0.285,Data!D$11)/42),LAMBDA(y,acc,FILTER(Data!A1:A12,(Data!E1:E12>1e-15),(y/123456789.123))))`: A619 Excel #CALC!, ours #VALUE!; cells (1): A619
- gen-m7/7307132bb8820f17.case.json A1324 `=MAP(Data!$D4,LAMBDA(v,((Data!D4*v)^BYROW(Data!B3:C11,LAMBDA(x,1e-15)))))`: A1324 Excel #CALC!, ours #VALUE!; cells (1): A1324
- gen-m7/9359f03972e94026.case.json A1705 `=SCAN(SUBSTITUTE(CHAR(9),FILTER(Data!A1:B12,(Data!E1:E12>0.5),Data!B$4),REPLACEB(Data!B6,8,Data!$C9,Data!D10)),Data!B5:D10,LAMBDA(acc,v,acc))`: A1705 Excel #CALC!, ours #VALUE!; cells (18): A1705:C1705, A1706:C1706, A1707:C1707, A1708:C1708, A1709:C1709, A1710:C1710
- gen-m7/b24ce74f08d3f91a.case.json A430 `=SCAN(Data!B8,Data!B6:E11,LAMBDA(acc,v,HSTACK(Data!D8:E8,Data!B8:C11)))`: A430 Excel #CALC!, ours #VALUE!; cells (24): A430:D430, A431:D431, A432:D432, A433:D433, A434:D434, A435:D435

## Acceptance criteria
- [ ] Visible parity: the 63 gen-m7 cells listed under Context have Excel's value, type, error and text: these 63 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Errors inside the step (T-2306).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
Done by T-2306: its `lambda_result_value` arm turns a multi-cell array step of MAP or SCAN into the one `#CALC!`. All 63 listed gen-m7 lines are gone from eval/baseline/gen-m7.txt (no change in this task's PR).
