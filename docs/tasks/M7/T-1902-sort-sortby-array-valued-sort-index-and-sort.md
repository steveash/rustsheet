---
id: T-1902
title: "SORT/SORTBY array-valued sort_index and sort_order (multi-key), SORTBY array order"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
SORT with an array sort_index/sort_order sorts by several keys, and the odd SORTBY array-order result, match Excel (probes/sort-error-args).

## Context
Probe `sort-error-args` (T-1850) cells H57 (`SORT(arr,{1,2},1)`), H69 (`SORT(arr,{1,2},{1,-1},TRUE)`) are multi-key sorts in Excel, H61/H65 (`SORT(arr,1,{1,-1})`) are #VALUE!, and H97 (`SORTBY(A1:A3,C1:C3,{1;-1;1})`) returns 1,0,0. The engine returns #N/IMPL! or a different value. Code: `fn_sort`/`fn_sortby` in crates/rustsheet-engine/src/functions/spill_functions.rs. Rules are in docs/excel-semantics/arrays.md.

## Acceptance criteria
- [ ] probes/sort-error-args has no failing cells in eval/baseline/probes.txt: H57:J59, H61, H65, H69:J71 and H97:H99 (all 23 known failures at 30bee011, spill members included). arrays.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
