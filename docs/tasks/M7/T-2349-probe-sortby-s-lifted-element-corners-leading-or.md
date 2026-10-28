---
id: T-2349
title: "Probe SORTBY's lifted-element corners: leading or mixed invalid sort_order entries, an error array or by_array"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-1839]
---

## Goal
Excel evidence for the SORTBY lifted-element cases that T-1839 implemented by analogy with UNIQUE, and the engine matching it.

## Context
From T-1839's Excel-fidelity review. arrays.md "UNIQUE's and SORTBY's lifted elements" marks these UNVERIFIED (A1:A3 = 3,1,2, C1:C3 = 2,1,3):
- `SORTBY(A1:A3,C1:C3,{2;1})`: ours `#VALUE!`, then 1, because an error element does not count as the kept first one (`Later` in functions/lift/driver.rs).
- `SORTBY(A1:A3,C1:C3,{1;-1;2})`: ours 1, empty, `#VALUE!`, the error per slot.
- `SORTBY(NA(),C1:C3,{1;-1})`: ours one scalar `#N/A`, because `fn_sortby` returns early on an error array. UNIQUE lifts an error array to every element (gen-m7/8044dce50616cfe9 A1434).
- `SORTBY(A1:A3,NA(),{1;-1})`: ours `#N/A` in every element.
Write a probe case per the excel-probe skill (ROWS, and per element ISBLANK, `&"|"`, TYPE and SUM, as probes/m7-unique-lift-elements does), fill it on excel-graph, and fix whatever differs in `fn_sortby`/`sortby_of`.

Also probe (from T-2348): SORTBY's uniform 1xN sort_order (`SORTBY(A1:A3,C1:C3,{1,1})`), with and without a second by_array/sort_order pair, and UNIQUE's error element past the first column (`UNIQUE(D1:D3,FALSE,{"a","a"})`); ours keeps SORTBY's top-lefts and UNIQUE's error.

## Acceptance criteria
- [ ] A probe case with these cells, filled from excel-graph; arrays.md updated (the UNVERIFIED lines resolved).
- [ ] Every cell of it passes, or each failing cell has its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
