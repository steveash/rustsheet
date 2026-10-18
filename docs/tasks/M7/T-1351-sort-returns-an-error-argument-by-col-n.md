---
id: T-1351
title: "SORT returns an error argument (by_col #N/A) before rejecting sort_index 0"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SORT returns the error of an error-valued argument (such as by_col) before it rejects an invalid sort_index or sort_order, as Excel does.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/c5d278a8662ef4b2 Calc!A39 `=SORT(Data!B11:E12,ROUND(0,7),Data!A3:C5,HLOOKUP(ADDRESS(19,19),Data!C8:C12,3))`:
Excel `#N/A` (by_col's HLOOKUP is `#N/A`), engine `#VALUE!` (sort_index 0 is rejected first: `SORT(Data!B11:E12,0,1,NA())` is
`#VALUE!`, with sort_index 1 it is `#N/A`). docs/excel-semantics/arrays.md "SORT argument errors come before validation".
Code: `fn_sort` in crates/rustsheet-engine/src/functions/spill_functions.rs. Evaluate all arguments and return the first
error (left to right) before validating values; check SORTBY the same way (unit test). T-1837 rewrites how SORT reads
sort_index/sort_order lists: rebase on it if it has landed.

## Acceptance criteria
- [ ] gen-m7/c5d278a8662ef4b2 A39 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: an error by_col with sort_index 0, an error sort_order with sort_index 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_sort` now evaluates sort_index, sort_order and by_col before validating values, so an error
by_col (or error sort_order/sort_index) wins over sort_index 0 / invalid sort_order. A multi-cell
sort_order that fails to coerce no longer pre-empts by_col's error. gen-m7 c5d278a8662ef4b2 A39 passes
(gen-m7 985/994 cells, gate=false, no baseline change). SORTBY and the order among several error
arguments stay UNVERIFIED (arrays.md).
