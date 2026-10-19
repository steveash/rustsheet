---
id: T-2012
title: "Match SUBTOTAL over TAKE of an array SWITCH case value and of IF over a one-cell INDIRECT condition (T-1984)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`SUBTOTAL(9,TAKE(SWITCH(1,{1,2},H1:H2,I1:I2),2))` spills a 2x2 `#VALUE!` and `SUBTOTAL(9,TAKE(IF(INDIRECT("K1"),H1:H2,I1:I2),2))` (K1 TRUE) gives 7, as Excel does.

## Context
Probe probes/subtotal-take-array-selector-b A10 (engine 14, Excel 2x2 `#VALUE!`) and A130 (engine 2x1 `#VALUE!`, Excel 7); see docs/excel-semantics/references.md. `lifts_to_values` (static_analysis/aggregate.rs) does not lift an array SWITCH case value, and treats a one-cell INDIRECT condition as an unknown-shape selector. Every other probe cell already matches.

## Acceptance criteria
- [ ] A10 and A130 of probes/subtotal-take-array-selector-b pass and no other cell of that case changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
