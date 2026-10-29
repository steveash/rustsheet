---
id: T-3027
title: "Probe SORTBY: error array with an invalid or uniform 1xN sort_order (which error wins)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for which error wins inside a SORTBY element when an error array meets an invalid or uniform 1xN sort_order, and the engine matching it.

## Context
From T-2349's Excel-fidelity review. Ours returns the array's error first (`sortby_of`, spill_functions.rs): `SORTBY(NA(),C1:C3,{2;1})` and `SORTBY(NA(),C1:C3,{1,1})` are `#N/A` in every element, UNVERIFIED (arrays.md "UNIQUE's and SORTBY's lifted elements"). Also probe a uniform 1xN order with a uniform second pair, `SORTBY(A1:A3,C1:C3,{1,1},A1:A3,1)` (ours 1 then empty since T-2349; it kept both before). Probe all three with the per-element cells of probes/sortby-lifted-element-corners, fill on excel-graph, fix `sortby_of`/the driver where Excel differs.

## Acceptance criteria
- [ ] A probe case with these cells, filled from excel-graph; arrays.md UNVERIFIED line resolved; every cell passes or has its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
