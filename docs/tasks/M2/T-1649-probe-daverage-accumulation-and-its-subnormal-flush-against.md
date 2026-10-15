---
id: T-1649
title: "Probe DAVERAGE accumulation and its subnormal flush against Excel"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DAVERAGE accumulates its sum and handles a subnormal result the way Excel does, with oracle evidence.

## Context
`fn_daverage` (`functions/database.rs`) sums with plain `+=` where AVERAGE uses `extended_sum` (numbers.md "Accumulators beyond SUM and AVERAGE": SUMIF, SUMIFS, DSUM, AVERAGEA round the total to a double after every addition) and, unlike `fn_average`, applies no `flush_subnormal` to the quotient. T-1618 covers SUMIF, SUMIFS, DSUM and AVERAGEA but not DAVERAGE. Reuse the T-1516 double-rounding-accumulators operand sets with a header row and criterion `>=0`, and a tiny-sum case for the subnormal quotient. Fix whichever differs.

## Acceptance criteria
- [ ] A probes-numeric case with oracle values for DAVERAGE over the T-1516 operand sets and a subnormal quotient; the engine matches, or `fn_daverage` uses `extended_sum` / `flush_subnormal`; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes-numeric/daverage-accumulation (excel-graph, 104 cells). DAVERAGE accumulates with per-addition double rounding (the engine missed by one ulp in all 8 operand sets) and flushes a subnormal quotient to 0 like AVERAGE (the engine kept it). `fn_daverage` now uses `extended_sum` and `flush_subnormal`. Rows A20:A24 use normal sums over larger counts and rows A26:A28 use kept subnormal operands built by subtraction (with the decision 0042 companion); both give 0. numbers.md updated.
