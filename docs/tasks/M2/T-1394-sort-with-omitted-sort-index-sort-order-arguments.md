---
id: T-1394
title: "SORT with omitted sort_index/sort_order arguments: SORT(arr,,) sorts, SORT(arr,,-1) is #SPILL!"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SORT with an omitted sort_index or sort_order matches Excel: `SORT({3;1;2},,)` sorts ascending and `SORT({3;1;2},,-1)` is #SPILL!.

## Context
probes/counta-empty-value D22-D24 (excel-graph, T-1294): `COUNTA(SORT({3;1;2},,))` = 3, `INDEX(SORT({3;1;2},,),1)` = 1, `SORT({3;1;2},,-1)` = #SPILL! (the engine gives 1, #VALUE!, #VALUE!). Probe further omitted-argument combinations first.

## Acceptance criteria
- [ ] probes/counta-empty-value D22, D23, D24 pass; further omitted-argument SORT/SORTBY combinations are probed and recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
