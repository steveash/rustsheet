---
id: T-1837
title: "SORT reads an array or range sort_index as a list of sort keys, with a sort_order of the same length"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SORT reads an array (or multi-cell range) sort_index as a list of sort keys, applied in order, and a sort_order with as many entries as sort_index, as in Excel; a sort_order list of another length is `#VALUE!`. Nothing of SORT lifts.

## Context
Split from T-1271. SPEC-0011 lists SORT's sort_index among the `Whole` parameters: it is read as one value, never lifted, so SORT needs no lifting-table entry and no part of T-1833's design. Today both an array sort_index and an array sort_order are `#N/IMPL!`.

Code: `fn_sort` and `get_sort_order` in crates/rustsheet-engine/src/functions/spill_functions.rs. Reuse SORTBY's multi-key comparison in the same file if it fits (directive 3); do not copy it. T-1351 (SORT returns an error argument before rejecting sort_index 0) changes the same function's argument handling: whichever lands second rebases, and this task keeps today's error order where the evidence below does not fix it.

Owned cells:
- probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
  - F10:H10 `SORT(A1:B3,{1;2})`: ROWS 3, COLUMNS 2, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
  - F11:H11 `SORT(A1:B3,1,{1;-1})`: `#VALUE!` in all three (ours `#N/IMPL!`).
- gen-m7/a0cbc75358b429b6 Calc!A12 `=SORT(CONCAT(..),Data!D1:E3,Data!A9:C12,(..=..))` (range sort_index and sort_order): Excel `#N/A`, ours `#N/IMPL!` (T-1275, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/a0cbc75358b429b6.case.json`). Read the full formula with `show` and record in arrays.md which check gives the `#N/A`.

## Acceptance criteria
- [x] probes/m7-argument-modes F10:H10 and F11:H11 pass.
- [x] gen-m7/a0cbc75358b429b6 A12 passes, with no new failing gen-m7 cell.
- [x] Their lines leave eval/baseline/ (`make eval-baseline`), with no new known failures.
- [x] Unit tests: two sort keys `{1;2}` and `{2,1}` (row and column lists) with ties broken by the second key; a sort_order list as long as the keys (`SORT(A1:B3,{1;2},{1;-1})`); a sort_order of a different length is `#VALUE!`; a scalar sort_index and sort_order are unchanged.
- [x] Any rule not already in docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes" is added there with its cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- SORT's argument-error order (T-1351).
- Lifting SORT's by_col (not probed).
- SORTBY (T-1839).

## Outcome
`fn_sort` (spill_functions.rs) reads sort_index and sort_order through `Model::sort_number_list`: a one-row or
one-column array or multi-cell range is a list, anything else one number. Keys apply in order, each tie broken by the
next (`first_unequal`, now shared with SORTBY's comparison). New probe probes/sort-index-key-ties (25 cells,
excel-graph) pins the rules the owned cells did not exercise: tie-breaking, a single sort_order for every key,
repeated keys allowed but more keys than columns (rows, by_col) `#VALUE!` (which also bounds the comparison's work),
a 2-D list and an error element `#VALUE!` as validation errors after later arguments' errors. A sort_order list of
more than one entry must be as long as sort_index. The argument-error order is unchanged. gen-m7/a0cbc75358b429b6
A12 already passed on main (its CONCAT array argument's `#N/A` comes first). Also completes T-1902 (marked done).
Eval delta: 26 probes cells fixed (m7-argument-modes F10:H11; sort-error-args H57:J59, H61, H65, H69:J71) plus the
new 25-cell case passing; 0 regressions; arrays.md updated.
