---
id: T-1516
title: "Probe double rounding in AVERAGEA, SUMIF, database sums and AVERAGE division by counts above 2"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with an oracle probe whether AVERAGEA, SUMIF(S), the database sums and AVERAGE with more than two values (the division by the count) round twice, and whether SUM of three or more terms rounds each running total twice (T-1499 extrapolated this from two-operand probes).

## Context
T-1499 made `+`, `-`, SUM and AVERAGE accumulate with `arithmetic::extended_sum`; these other accumulators still use the plain `+` or `/` and are unprobed (docs/excel-semantics/numbers.md "Division rounds twice").

## Acceptance criteria
- [ ] A probes-numeric case with oracle values for each; the engine matches or a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
