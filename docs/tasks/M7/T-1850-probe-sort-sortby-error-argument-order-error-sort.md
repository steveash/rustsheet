---
id: T-1850
title: "Probe SORT/SORTBY error-argument order: error sort_order with sort_index 0, several error arguments, SORTBY"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior for SORT and SORTBY with error arguments beside invalid values is evidenced and implemented.

## Context
T-1351 made SORT return an argument error before validating sort_index/sort_order, evidenced by one oracle cell only (gen-m7/c5d278a8662ef4b2 A39). Unverified: which error wins among several, an error sort_order with sort_index 0, and SORTBY. See docs/excel-semantics/arrays.md "SORT argument errors come before validation"; code: `fn_sort`/`fn_sortby` in crates/rustsheet-engine/src/functions/spill_functions.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cases cover the shapes above (also `SORT(arr,0,1)`, `SORT(arr,1,0)` and a multi-cell sort_order with valid sort_index and by_col) with oracle values; arrays.md updated; engine matches (or follow-up filed).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
