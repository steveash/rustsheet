---
id: T-1446
title: "Probe omitted-argument SORT/SORTBY combinations (sort_index, sort_order, by_col)"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SORT and SORTBY match Excel for every omitted-argument combination (sort_index, sort_order, by_col, SORTBY by_array/order pairs).

## Context
T-1394 made `SORT(arr,,)` use defaults (probes/counta-empty-value D22-D24) but other combinations, e.g. `SORT(arr,,,)` and SORTBY with empty arguments, are unprobed. Code: functions/spill_functions.rs.

## Acceptance criteria
- [ ] Probe cases cover omitted-argument combinations for SORT and SORTBY, with oracle values and the findings in docs/excel-semantics/arrays.md; the engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/sort-omitted-args` (75 cells, excel-graph) covers SORT/SORTBY omitted-argument combinations. SORT already matched; SORTBY with an omitted `sort_order` was `#VALUE!`, now ascending (shared `get_sort_order` helper). Three SORTBY forms are refused by Graph and left as a desktop question; findings in docs/excel-semantics/arrays.md. Eval delta: probes known failures 2848 -> 2848, one new fully passing case. Follow-up: T-1482.
