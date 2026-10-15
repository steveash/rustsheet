---
id: T-1648
title: "Probe the final division of DAVERAGE for double rounding"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DAVERAGE divides its sum by the count the way Excel does (twice-rounded, as AVERAGE), with oracle evidence.

## Context
T-1636 showed AVERAGEA, AVERAGEIF(S) and SUBTOTAL(1/101) double-round the final division (probes-numeric/average-family-large-count-division). `DAVERAGE` (`functions/database.rs`, `sum / count as f64`) was not probed and still divides once. Reuse the T-1636 columns (a single-column database with a header, criterion `>=0`). Change it to `arithmetic::extended_quotient` only if the oracle double-rounds.

## Acceptance criteria
- [ ] A probes-numeric case with oracle values for DAVERAGE at n = 2051, 2999, 4095; the engine matches, or the division uses `extended_quotient`; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
