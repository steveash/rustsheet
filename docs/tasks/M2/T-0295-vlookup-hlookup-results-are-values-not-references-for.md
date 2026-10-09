---
id: T-0295
title: "VLOOKUP/HLOOKUP picks reach aggregates as references only when the index argument is a cell reference (follow-up of T-0292)"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Aggregates (COUNT, SUM, MAX, ...) see the cell VLOOKUP or HLOOKUP picks as a reference only when the row/column index argument is a cell reference (parenthesized too); with a literal or computed index it is a plain value, as Excel does.

## Context
T-0292 probe `probes/lookup-pick-aggregates` (excel-graph) and docs/excel-semantics/references.md: the rule above, incl. why gen-m2/60a543744fc4a8f7 A200 (-2) must keep passing. T-0235 made `vlookup_impl`/`hlookup_impl` pass the pick as a reference always (functions/util.rs `evaluate_aggregate_arg` arms). Failing probe cells are the known failures in eval/baseline/probes.txt. ISREF stays FALSE; INDEX and array-table cases keep passing.

## Acceptance criteria
- [ ] Every cell of `probes/lookup-pick-aggregates` passes and its baseline lines are removed; gen-m2/60a543744fc4a8f7 A200 keeps passing.
- [ ] Comments citing gen-m2 A200 as reference evidence (lookup_and_reference/mod.rs, lookup_table.rs) and the test at test/test_intersection_operator.rs:379 are rewritten to the probe evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
