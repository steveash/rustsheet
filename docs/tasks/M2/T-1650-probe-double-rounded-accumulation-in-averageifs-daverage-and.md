---
id: T-1650
title: "Probe double-rounded accumulation in AVERAGEIFS, DAVERAGE and AVEDEV (T-1618 follow-up)"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence on whether AVERAGEIFS (and AVERAGEIF), DAVERAGE and AVEDEV round each addition twice like SUM, AVERAGEA, SUMIF and DSUM.

## Context
T-1618 moved SUMIF, SUMIFS, DSUM and AVERAGEA onto `extended_sum`; AVERAGEIFS (`average_ifs` in functions/statistical/if_ifs.rs), DAVERAGE (functions/database.rs) and AVEDEV (functions/statistical/count_and_average.rs) still add with plain `+=`. Extend probes-numeric/double-rounding-accumulators with the same 8 operand sets (the divisions are exact when the count is a power of two, or use the extended_quotient expectations) via the excel-probe skill, then switch the engine if Excel double-rounds.

## Acceptance criteria
- [ ] Cells added to probes-numeric/double-rounding-accumulators with oracle values; numbers.md updated; engine matches or a task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed probes-numeric/double-rounding-averageifs (56 cells, excel-graph): Excel rounds each addition twice in AVERAGEIFS, AVERAGEIF and AVEDEV (mean sum and deviation sum). Engine now uses `extended_sum`; all 56 cells pass. DAVERAGE was already covered by T-1649. Recorded in numbers.md.
