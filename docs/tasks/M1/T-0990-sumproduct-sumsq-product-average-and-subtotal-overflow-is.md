---
id: T-0990
title: "SUMPRODUCT, SUMSQ, PRODUCT, AVERAGE and SUBTOTAL overflow is #NUM!"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMPRODUCT, SUMSQ, PRODUCT, AVERAGE and SUBTOTAL return #NUM! when a sum, product or square overflows, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Aggregate overflow), evidence probes/aggregate-overflow (T-0192, excel-graph 2026-10-06). Engine returns inf/NaN text: B11-B13 (SUMPRODUCT), B21-B23 (SUMSQ), B14-B19 (AVERAGE, PRODUCT incl. a later 0 or 1E-200 factor: still #NUM!), B24-B26 (SUBTOTAL) and the C ERROR.TYPE cells. Reuse the SUM overflow check (T-0185).

## Acceptance criteria
- [ ] probes/aggregate-overflow B11-B26 and C11-C26 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
SUMPRODUCT, SUMSQ, PRODUCT, AVERAGE and SUBTOTAL 1/6/9 return `#NUM!` on overflow via `CalcResult::finite_number_or_num` on the final result; unit test `overflowing_aggregates_are_num`. Eval: probes known failures 257 -> 225 (fixed 32), regressions 0.
