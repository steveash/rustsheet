---
id: T-1322
title: "GROUPBY core: grouping, ordering, LAMBDA functions, headers, totals (SPEC-0006 step 1)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1276]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY with a LAMBDA function computes the probed results for grouping, ordering, headers and grand totals.

## Context
SPEC-0006 Design steps 1-3, 5-7; evidence probes/groupby-pivotby sheet G and docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY". Build the shared `Grouping` core in functions/lookup_and_reference/groupby.rs, reusing `cell_key` and `sort_key_cmp` (spill_functions.rs, made pub(crate)) and `call_lambda_with_values`. Cover keys of one or more columns, case folding, mixed-type order with blank last (shown as 0), source-order function inputs, total row over all values, field_headers 0-3 plus auto-detect, total_depth 0/1/-1, `""` fillers, several value columns, error values, and a LAMBDA returning an array giving `#VALUE!` per cell. Eta names wait for T-1273 / T-1325.

## Acceptance criteria
- [ ] The LAMBDA forms of the G-sheet anchors that need only these features pass (rewrite them as LAMBDA cases in a new probe if the eta names block scoring), and probes/m7-argument-modes G34/H34 pass; their baseline entries are removed.
- [ ] Unit tests cover the grouping order (mixed types, case folding, blank last) and the `""` filler vs the 0 blank key.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
