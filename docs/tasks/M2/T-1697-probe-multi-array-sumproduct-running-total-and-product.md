---
id: T-1697
title: "Probe multi-array SUMPRODUCT running total and product rounding"
milestone: M2
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's multi-array SUMPRODUCT (running total and per-term products) is probed and the engine matches it bit for bit.

## Context
T-1676 switched SUMPRODUCT's running total to `extended_sum` on single-array evidence only (probes-numeric/double-rounding-avedev-deviations). Multi-array totals and the plain `acc * a[i][j]` products in `fn_sumproduct` are unprobed; see numbers.md "double rounding".

## Acceptance criteria
- [x] Probe case with operand sets whose multi-array products and sums differ under single vs double rounding; engine passes or gaps are filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes-numeric/double-rounding-sumproduct-multi (110 cells, excel-graph): multi-array SUMPRODUCT products round twice like `*` and its running total like SUM. The engine's plain products missed by one ulp in the 8 two-array pairs (3-array and running-total cells already matched); `fn_sumproduct` now multiplies with `extended_product`. Recorded in numbers.md.
