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
Split from T-2360. probes/t2360-array-lambda-holidays G!A12, G!B3, G!C4, G!C11, G!D3, G!D12 (excel-graph, 2026-10-23):
`INDEX(GROUPBY(D90!A1:D12,..),r,c)` over a blank key cell is `1|` (TYPE 1, empty text); ours `1|0`. arrays.md (paragraph "A LAMBDA
array result is shown as `#VALUE!`...") records it; the GROUPBY section already says blank values inside LAMBDA arguments are true
blanks. Code: the GROUPBY/PIVOTBY key column build (crates/rustsheet-engine/src/functions/, search `GROUPBY`).
Handed over from T-2578 (excel-graph, 2026-10-24): gen-m9/8d3588d8df0812df Calc!A240
`AVERAGEA(GROUPBY(Data!B1:E12,Data!B1:D12,_xleta.PRODUCT,0,2),..)` differs only in its 9 blank keys (probes/
t2578-groupby-product-subtotals: COUNT 91, ours 100); probes/t2578-pivotby-total-error-scope P!D49-D65 show blank PIVOTBY
row and column keys as `""` through MAP (ours "0"). Note `shown_key` (grouping.rs) cites G!A1 and groupby-pivotby-followup
G!A821 for 0: those are the spilled cells, where a true blank element displays as 0.

## Acceptance criteria
- [ ] The six G! known-failure lines above disappear from eval/baseline/probes.txt, no new known failures.
- [ ] gen-m9/8d3588d8df0812df Calc!A240 and the eleven probes/t2578-* blank-key lines disappear from the baselines.
- [ ] Unit test: `TYPE(INDEX(GROUPBY(..blank key..),r,1))&INDEX(..)` is "1".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Nested-array cells (T-2459).
