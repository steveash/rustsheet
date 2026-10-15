---
id: T-1649
title: "Probe DAVERAGE accumulation and its subnormal flush against Excel"
milestone: M2
lane: research
status: todo
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
