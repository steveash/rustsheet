---
id: T-0990
title: "SUMPRODUCT, SUMSQ, PRODUCT, AVERAGE and SUBTOTAL overflow is #NUM!"
milestone: M1
lane: functions-math
status: todo
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
- [ ] probes/aggregate-overflow B11-B26 and C11-C13 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
