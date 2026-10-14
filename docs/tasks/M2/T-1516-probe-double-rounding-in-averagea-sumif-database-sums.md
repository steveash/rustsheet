---
id: T-1516
title: "Probe double rounding in AVERAGEA, SUMIF, database sums and AVERAGE division by counts above 2"
milestone: M2
lane: research
status: todo
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

## Notes from an unfinished attempt (session ran out of time; no probe case was written)
- Operands for a 3-term SUM where only the second addition discriminates are easy to find by exact
  simulation: a double, plus two doubles scaled by 2^-11..2^-50, rejecting draws where the first
  partial sum already differs (about 1 draw in 7000 passes; use `fractions.Fraction`, or the fit lab's
  exact arithmetic in `tools/`). Put them in cells and reference them through SUM(range), SUMIF,
  SUMIFS, DSUM (database in a second sheet), AVERAGEA and `a+b+c`.
- AVERAGE's division by a count of 3, 5 or 7 cannot be probed: for a small odd count n the binary
  expansion of k/n has no run of 11 zero bits after the cut, so the 64-bit rounding never lands on a
  midpoint and double rounding equals single rounding. A discriminating divisor needs n of a few
  thousand (random search at n = 1025 found none in 20 minutes); record that bound in numbers.md
  rather than burning time on small counts.
