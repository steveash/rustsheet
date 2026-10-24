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

## Acceptance criteria
- [ ] The six G! known-failure lines above disappear from eval/baseline/probes.txt, no new known failures.
- [ ] Unit test: `TYPE(INDEX(GROUPBY(..blank key..),r,1))&INDEX(..)` is "1".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Nested-array cells (T-2459).
