---
id: T-0421
title: "Fast path for the twice-rounded division: skip the u128 divide away from a double midpoint"
milestone: M3
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The operator `/` costs about as much as a plain hardware division in the common case, while keeping
Excel's twice-rounded result (docs/excel-semantics/numbers.md "Division rounds twice").

## Context
T-0221 performance review: `arithmetic::extended_quotient` does a software u128 `/` and `%` on
every division. The double rounding can change the result only when the exact quotient lies within
half an extended ulp of a double midpoint. A residual `x - plain*y` (exact through `mul_add`, or
an equivalent cheap test) decides that, so almost every division can return `plain` at once.
Check that `mul_add` is not itself a slow software call on the release target before using it.

## Acceptance criteria
- [ ] The fast path returns the same bits as the u128 path: a unit test over many random and
  midpoint-adjacent operands, plus the probes-numeric/power-reciprocal-rounding cells.
- [ ] A micro-benchmark or timing note shows the before/after cost per division.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which operations round twice (T-1496).
