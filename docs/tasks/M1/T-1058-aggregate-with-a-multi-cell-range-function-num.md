---
id: T-1058
title: "AGGREGATE with a multi-cell range function_num or options"
milestone: M1
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`AGGREGATE(F1:F2,0,E1:E6)` and `AGGREGATE(9,F1:F2,E1:E6)` (a multi-cell range as function_num or options) give Excel's values and spill shape.

## Context
Found by T-1038's review. Since T-1038, an array function_num, options or k lifts element-wise
(docs/excel-semantics/numbers.md "AGGREGATE"). A multi-cell range there is still read as a scalar
through `cast_to_number`, so the engine gives `#N/IMPL!`. In dynamic-array Excel it probably lifts like
an array, but nothing shows it either way. Probe first: a range in a row and in a column, a
single cell, and a range next to an array k. Then extend `fn_aggregate`
(crates/rustsheet-engine/src/functions/aggregate.rs, the `ranges` stash) and
`aggregate_static_result` (expressions/parser/static_analysis/aggregate.rs).

## Acceptance criteria
- [ ] A probe case holds oracle values (and neighbouring spill cells) for those forms, and it passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
