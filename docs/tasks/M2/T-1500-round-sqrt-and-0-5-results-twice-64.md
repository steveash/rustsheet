---
id: T-1500
title: "Round SQRT and ^0.5 results twice (64-bit, then double)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SQRT and `^0.5` round the exact result to a 64-bit significand and then to a double, as Excel does.

## Context
T-1496 probed this: probes-numeric/double-rounding-ops A65-A80 (`SQRT(x)` and `x^0.5`) are operand sets where single and double rounding differ, and Excel (excel-graph) returns the double-rounded value in every cell. The engine returns the singly-rounded value. Semantics: docs/excel-semantics/numbers.md "Division rounds twice". Helper to reuse or extend: `arithmetic::extended_quotient`. Code path: `arithmetic::sqrt`/the `^0.5` path in `non_integer_power`.

## Acceptance criteria
- [ ] probes-numeric/double-rounding-ops A65-A80 pass; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
