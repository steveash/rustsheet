---
id: T-1499
title: "Round +, - SUM and AVERAGE results twice (64-bit, then double)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`+`, `-`, SUM and AVERAGE round the exact result to a 64-bit significand and then to a double, as Excel does.

## Context
T-1496 probed this: probes-numeric/double-rounding-ops A33-A64 (`a+b`, `SUM(a,b)`, `AVERAGE(a,b)` (sum rounded twice, then halved) and `a-b` with operands whose exponents differ by 12-52) are operand sets where single and double rounding differ, and Excel (excel-graph) returns the double-rounded value in every cell. The engine returns the singly-rounded value. Semantics: docs/excel-semantics/numbers.md "Division rounds twice". Helper to reuse or extend: `arithmetic::extended_quotient`. Code path: the addition/subtraction paths and the SUM/AVERAGE accumulators.

## Acceptance criteria
- [ ] probes-numeric/double-rounding-ops A33-A64 pass; `rustsheet-eval show probes-numeric/double-rounding-ops.case.json` lists no failure in them, and no gated suite regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
