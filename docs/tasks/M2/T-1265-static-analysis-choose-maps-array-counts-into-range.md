---
id: T-1265
title: "Static analysis: CHOOSE maps Array counts into Range extents"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`StaticResult::Range` holds one convention everywhere (extents minus one, a single cell is
`Range(0, 0)`), so no caller compares a range shape off by one.

## Context
T-0296 found AGGREGATE comparing a `Range` against `(1, 1)` as if it held counts (a 2x2 range
function_num reached a scalar cell, debug-assert panic), and documented `Range` as extents minus one.
`Function::Choose` in `expressions/parser/static_analysis.rs` still turns `Array(n, m)` (counts) into
`Range(n, m)`, and `broadcast_static_result` takes the max of counts and extents together. Reviewers
of T-0296 flagged it. Audit every producer and consumer of `StaticResult::Range` (legacy-import `@`
placement at the `Range(_, _)` matches, AGGREGATE, SUBTOTAL/AREAS) and either convert CHOOSE or
switch `Range` to counts, with tests showing no export or spill change (or the evidence for one).

## Acceptance criteria
- [ ] One documented convention for `StaticResult::Range`, used by every producer.
- [ ] Unit tests for `AGGREGATE(CHOOSE(1,A1:B2),0,E1:E6)` and a legacy CHOOSE import keep Excel's results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Tracking shapes of names and tables.
