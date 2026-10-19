---
id: T-2012
title: "Probe RANK order read from a text cell and the number-vs-order error order"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
RANK's `order` read from a cell holding text, and the error order between `number` and `order`, follow
excel-graph evidence instead of T-1999's assumption.

## Context
T-1999 (`statistical/rank_eq_avg.rs`, `rank_element`) makes any text `order` `#VALUE!`: typed `"1"`
(probes/rank-direct-boolean H10) and array text (probes/stat-test-lift P33) are evidenced, but a text
cell is not, and for `number` a text cell "4" coerces (H8). It also reads `number` before `order`
with no evidence of which error wins. statistics.md "RANK, FORECAST, PROB, Z.TEST and T.TEST" says
"Not probed: an `order` from a cell holding text". Probe (excel-probe skill) in a new probes case:
`RANK(4,A1:A8,K2)` with K2 a text "1" and with K2 "x", `RANK(4,A1:A8,M1:M2)` with a text cell in
M1:M2, `RANK("x",A1:A8,1/0)`, `RANK(NA(),A1:A8,"x")`, and RANK.AVG of the first.

## Acceptance criteria
- [ ] A new probe case with excel-graph values; statistics.md records the findings (drop "Not probed").
- [ ] The kernel matches every probed cell, with eval baseline updated if anything changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- RANK's `ref` reading (T-1999, T-1303).
