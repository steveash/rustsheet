---
id: T-2460
title: "GROUPBY/PIVOTBY: a blank key stays a true blank in the result, not 0"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A blank row-field key in GROUPBY and PIVOTBY results is a true blank (TYPE 1, `&""` is ""), as Excel has it, not the number 0.

## Context
Also owns (T-2623): gen-m9/d37313406b9f6e55.case.json Calc!A182 `=TRIMMEAN(_xlfn.GROUPBY(Data!E1:E12,Data!E1:E12,LAMBDA(z,ROUND(2,5)),0),(Data!C12-NPV(100,(-0.5))))`:
Excel 8230453.917639333, ours 7716050.547786875; the GROUPBY's blank-key row shows `""` in Excel and 0 in ours
(probes/t2475-stat-subexpressions P!A530, A531), and TRIMMEAN then counts our 0.
Split from T-2360. probes/t2360-array-lambda-holidays G!A12, G!B3, G!C4, G!C11, G!D3, G!D12 (excel-graph, 2026-10-23):
`INDEX(GROUPBY(D90!A1:D12,..),r,c)` over a blank key cell is `1|` (TYPE 1, empty text); ours `1|0`. arrays.md (paragraph "A LAMBDA
array result is shown as `#VALUE!`...") records it; the GROUPBY section already says blank values inside LAMBDA arguments are true
blanks. Code: the GROUPBY/PIVOTBY key column build (crates/rustsheet-engine/src/functions/, search `GROUPBY`).
Handed over from T-2578 (excel-graph, 2026-10-24): gen-m9/8d3588d8df0812df Calc!A240
`AVERAGEA(GROUPBY(Data!B1:E12,Data!B1:D12,_xleta.PRODUCT,0,2),..)` differs only in its 9 blank keys (probes/
t2578-groupby-product-subtotals: COUNT 91, ours 100); probes/t2578-pivotby-total-error-scope P!D49-D65 show blank PIVOTBY
row and column keys as `""` through MAP (ours "0"). Note `shown_key` (grouping.rs) cites G!A1 and groupby-pivotby-followup
G!A821 for 0: those are the spilled cells, where a true blank element displays as 0.

Also owned (T-2625, 2026-10-24): probes/t2625-nested-consumers P!A13, A18, A19, A23, A24, A26 (COUNT over a PIVOTBY with several
key fields) and A28:A31 (its key column and column-key rows as text: Excel shows a blank key as "" in the row keys and in each
column-key row, ours 0), and gen-m9/e85daedce5c2675e Calc!A35 (`BESSELJ(COUNT(PIVOTBY(..)),..)`, the same COUNT plus 2).

Exit owners (decision 0096; the 2026-10-27 planner run names each cell the ranges above hold):
- probes/t2578-pivotby-total-error-scope P!D50, P!D51, P!D53, P!D55, P!D62, P!D64
- probes/t2578-groupby-product-subtotals P!A3, P!A6
- probes/t2625-nested-consumers P!A29, P!A30

## Acceptance criteria
- [ ] The six G! known-failure lines above disappear from eval/baseline/probes.txt, no new known failures.
- [ ] gen-m9/8d3588d8df0812df Calc!A240 and the eleven probes/t2578-* blank-key lines disappear from the baselines.
- [ ] Unit test: `TYPE(INDEX(GROUPBY(..blank key..),r,1))&INDEX(..)` is "1".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Nested-array cells (T-2459).
