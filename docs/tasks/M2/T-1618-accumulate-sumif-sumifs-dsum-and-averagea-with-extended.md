---
id: T-1618
title: "Accumulate SUMIF, SUMIFS, DSUM and AVERAGEA with extended_sum (they round once, Excel twice)"
milestone: M2
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMIF, SUMIFS, DSUM and AVERAGEA add their terms with `arithmetic::extended_sum` (each addition rounded twice, 64-bit then double), like SUM and `+`, so probes-numeric/double-rounding-accumulators passes.

## Context
Found by T-1516: Excel double-rounds every addition in these accumulators (docs/excel-semantics/numbers.md, "Division rounds twice"). The engine is one ulp off in the cells SUMIF, SUMIFS, DSUM and AVERAGEA of all 8 operand sets (32 cells; `make eval-show CASE=probes-numeric/double-rounding-accumulators.case.json`). SUM and `+` already use `extended_sum` (T-1499); `rg` for the plain `+=` in the SUMIF/database/AVERAGEA accumulators and share the helper.

## Acceptance criteria
- [x] The 32 failing cells of probes-numeric/double-rounding-accumulators pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
SUMIF, SUMIFS (`sum_ifs`), DSUM and AVERAGEA now add with `extended_sum` like SUM and AVERAGE. probes-numeric/double-rounding-accumulators: 104/104 cells pass (the 32 failing cells fixed). `probes-numeric` has no baseline file, so no baseline change. Not changed (unprobed): AVERAGEIFS, DAVERAGE and AVEDEV still add with plain `+=`.
