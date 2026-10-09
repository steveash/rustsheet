---
id: T-0295
title: "VLOOKUP/HLOOKUP results are values, not references, for aggregates (follow-up of T-0292)"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Aggregates (COUNT, SUM, MAX, ...) treat the cell VLOOKUP or HLOOKUP picks from a range table as a plain value, as Excel does.

## Context
T-0292 probe `probes/lookup-pick-aggregates` (excel-graph) shows the picked cell is a value, not a reference (ISREF FALSE; COUNT counts a picked FALSE; SUM coerces picked "5"; MAX of picked text is #VALUE!). T-0235 made `vlookup_impl`/`hlookup_impl` pass it as a reference (functions/util.rs `evaluate_aggregate_arg` arms). Failing cells: D1-D3, D13-D15. See docs/excel-semantics/references.md. Keep ISREF FALSE and the INDEX/array cases passing.

## Acceptance criteria
- [ ] Cells D1-D3 and D13-D15 of `probes/lookup-pick-aggregates` pass; the gen-m2 cases touching VLOOKUP/HLOOKUP do not regress.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
