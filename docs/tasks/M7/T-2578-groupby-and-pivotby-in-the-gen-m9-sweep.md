---
id: T-2578
title: "GROUPBY and PIVOTBY in the gen-m9 sweep: eta AVERAGE/PRODUCT with multi-column fields, and aggregates over LAMBDA(x,x) results (7 cells)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2459]
campaign: visible
---

## Goal
The GROUPBY and PIVOTBY calls of these gen-m9 cells give Excel's result shape and elements, so the enclosing functions match.

## Context
From T-2436's probe of the untraced gen-m9 cells. arrays.md "GROUPBY and PIVOTBY in the gen-m9 sweep (T-2436)". The eta AVERAGE/PRODUCT cells need no LAMBDA model: find why ours is `#VALUE!` (multi-column row or column fields, a values range wider than one column, total_depth 2) and why the GROUPBY product differs (compare with an INDEX/ROWS/COLUMNS probe of the result; the spill is blocked in the sweep). The LAMBDA(x,x) cells (MAX, HARMEAN, AVERAGE `#N/A`, COUNT 142) follow T-2459's nested-array model. Code: functions/lookup_and_reference/groupby.rs and pivotby.rs.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): ours fails or differs on GROUPBY/PIVOTBY calls Excel computes: `PIVOTBY(Data!B1:E12,Data!B1:D12,Data!D1:E12,_xleta.AVERAGE)` is 16x30 and `PIVOTBY(Data!B1:B12,Data!B1:C12,Data!B1:C12,_xleta.AVERAGE,1)` 13x25 (ROWS/COLUMNS; the spill is blocked, so `#SPILL!` as a cell; ours `#VALUE!`), `AVERAGEA(GROUPBY(Data!B1:E12,Data!B1:D12,_xleta.PRODUCT,0,2),..)` -4.454221888962187E+22 (ours -4.161608772169051E+22; GROUPBY 20x7); and over a LAMBDA(x,x) or LAMBDA(acc,acc-acc) function the result's elements differ: `MAX(..,PIVOTBY(..,LAMBDA(z,z)),2.5)`, `HARMEAN(..,PIVOTBY(..,LAMBDA(acc,(acc-acc))),..)` and `AVERAGE(..,GROUPBY(..,LAMBDA(w,w),..),..)` are `#N/A` (ours `#VALUE!`), `COUNT(PIVOTBY(..,LAMBDA(y,y)),..)` 142 (ours 117). The LAMBDA cases depend on T-2459's nested-array model.
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 7 known-failure lines in eval/baseline/gen-m9.txt, 7 formulas.
Examples:
- gen-m9/899f9555e67da2d6 Calc!A311 `=EXPONDIST(WEIBULL(AVERAGE(Data!B11:B11),10000000001,Data!B3,1),(STDEV(1.005,Data!B8:B10,Data!E8:E10)*(Data!D$3*1)),_xlfn.PIVOTBY(Data!B1:E12,Data!B1:D12,Data!D1:E12,_xleta.AVERAGE))`: Excel #SPILL!, ours #NUM!; inner `_xlfn.PIVOTBY(Data!B1:E12,Data!B1:D12,Data!D1:E12,_xleta.AVERAGE)` Excel #SPILL!, ours #VALUE!
- gen-m9/5b6cc12a94f7bf14 Calc!A90 `=BESSELJ(SEQUENCE(2,DAYS(24664,(-0.5))),_xlfn.PIVOTBY(Data!B1:B12,Data!B1:C12,Data!B1:C12,_xleta.AVERAGE,1))`: Excel #SPILL!, ours #NUM!; inner `_xlfn.PIVOTBY(Data!B1:B12,Data!B1:C12,Data!B1:C12,_xleta.AVERAGE,1)` Excel #SPILL!, ours #VALUE!
- gen-m9/8d3588d8df0812df Calc!A240 `=AVERAGEA(_xlfn.GROUPBY(Data!B1:E12,Data!B1:D12,_xleta.PRODUCT,0,2),{0.02,0.027},Data!B7:B7)`: Excel -4.454221888962187e22, ours -4.161608772169051e22
- gen-m9/06b4e8f5de409d5e Calc!A366 `=CHIINV(MAX(Data!B4:B8,_xlfn.PIVOTBY(Data!C1:E12,Data!C1:E12,Data!C1:E12,LAMBDA(z,z)),2.5),(IF(FALSE,0,42)+QUARTILE(Data!E6,4)))`: Excel #N/A, ours #VALUE!; inner `MAX(Data!B4:B8,_xlfn.PIVOTBY(Data!C1:E12,Data!C1:E12,Data!C1:E12,LAMBDA(z,z)),2.5)` Excel #N/A, ours #VALUE!
Cells:
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A311
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A90
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A240
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A366
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A285
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A381
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A35

## Acceptance criteria
- [ ] Not met as written: 2 of 7 fixed; the other 5 handed over with evidence to T-2460 and T-2593 (see Outcome), and 11 blank-key probe lines of T-2460 baselined. Visible parity (gen-m9, T-2436): these 7 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Outcome
Probed which hidden PIVOTBY Total cells make the result `#VALUE!` (probes/t2578-pivotby-hidden-total-slot, checking a 256-layout decode run but not committed,
plus 3-value and two-field checks; t2578-pivotby-total-error-scope; excel-graph 2026-10-24): Excel computes the Total cell of
value column `v` in a row only when the row has a record under the column line at value-column position `G + v`, line
`(G + v) div V`. `check_totals` (pivotby.rs) now gates each checked Total slot by that line (`checked_totals`), so the two
eta-AVERAGE PIVOTBY calls (899f9555e67da2d6 A311, 5b6cc12a94f7bf14 A90) are arrays of Excel's shape.
The other five owned cells have other causes, handed over with evidence: 8d3588d8df0812df A240 is only the GROUPBY's blank keys
(probes/t2578-groupby-product-subtotals) → T-2460; the four LAMBDA(x,x) cells (06b4e8f5de409d5e A366, 68d8ffb69ff739d8 A285,
7c20351562a2d71d A381, e85daedce5c2675e A35) need nested arrays → T-2593.
Eval delta: gen-m9 known failures 6865 -> 6863; probes 2819 -> 2818 (12 fixed, among them the t2436 PIVOTBY sub-expressions;
three new cases, whose only failures are 11 blank-key cells of T-2460); no regressions.
