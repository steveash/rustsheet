---
id: T-1498
title: "Round the product of * and PRODUCT twice (64-bit, then double)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
the operator `*` and PRODUCT round the exact result to a 64-bit significand and then to a double, as Excel does.

## Context
T-1496 probed this: probes-numeric/double-rounding-ops A1-A16 (`a*b` and `PRODUCT(a,b)` of two full-significand operands) are operand sets where single and double rounding differ, and Excel (excel-graph) returns the double-rounded value in every cell. The engine returns the singly-rounded value. Semantics: docs/excel-semantics/numbers.md "Division rounds twice". Helper to reuse or extend: `arithmetic::extended_quotient`. Code path: the multiplication in `arithmetic`.

## Acceptance criteria
- [ ] probes-numeric/double-rounding-ops A1-A16 pass; `rustsheet-eval show probes-numeric/double-rounding-ops.case.json` lists no failure in them, and no gated suite regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
