---
id: T-1242
title: "INDEX of a single value or a 1x1 array returns that value"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
INDEX of a single value or a 1x1 array with row/column 1 returns that value, as Excel does.

## Context
probes/lookup-argument-modes (excel-graph, 2026-10-09, T-1238): `INDEX(LOOKUP(2,{1,2,3},{4,5,6}),1,1)` = 5,
`INDEX(XMATCH(2,{1,2,3}),1,1)` = 2, `INDEX(ROWS({1;2;3}),1,1)` = 3, `INDEX(AREAS(A1:B3),1,1)` = 1; the engine gives
`#VALUE!` (H18, H19, H24, H28, H30, H31, H39). Probe what INDEX of a scalar does with other indexes (`INDEX(5,2,1)`,
`INDEX(5,0,0)`, `INDEX("a",1)`) with the excel-probe skill first. Code: `fn_index` (functions/lookup_and_reference.rs).

## Acceptance criteria
- [ ] probes/lookup-argument-modes H18, H19, H24, H28, H30, H31, H39 pass, plus the new probe cells; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
H18, H19, H24, H28, H30, H31 and H39 already pass on `main`. Added probe `probes/index-scalar-indexes`
(23 cells, excel-graph) for INDEX of a scalar or 1x1 array with other indexes; the engine matched every
cell. Rules in lookup.md "INDEX of a scalar or a 1x1 array". No engine change.
