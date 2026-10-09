---
id: T-1271
title: "M7 array functions lift their scalar arguments; SORT reads sort_index and sort_order as lists"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
The scalar arguments of SEQUENCE, TAKE, DROP, EXPAND, MAKEARRAY, WRAPROWS, WRAPCOLS, TOCOL, UNIQUE and SORTBY lift over a range and an array (one call per element) as in Excel, and SORT reads an array sort_index as a list of sort keys with a sort_order of the same length.

## Context
T-1083's probe probes/m7-argument-modes (excel-graph, 2026-10-09; docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"). Failing rows (F/G/H cells): 1-3 SEQUENCE (a range gives one row, an array `#N/IMPL!`), 4-5 TAKE, 6 DROP (its first lifted element is `#SPILL!` in Excel), 7 EXPAND, 8 WRAPROWS, 9 TOCOL, 10-11 SORT (`SORT(A1:B3,{1;2})` is 3x2; `SORT(A1:B3,1,{1;-1})` is `#VALUE!`), 12 UNIQUE, 23 MAKEARRAY, 26 SORTBY, 42 WRAPCOLS pad_with. Reuse the existing lift helpers (T-1241 does the same for the M6 lookup functions; check whether it landed a shared helper first). The functions live in functions/lookup_and_reference and spill_functions.rs. Split per function family if it outgrows size M.

## Acceptance criteria
- [ ] The listed probes/m7-argument-modes cells pass (H cells that need T-1242, INDEX of a single value, excepted); baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
