---
id: T-1499
title: "Round +, - SUM and AVERAGE results twice (64-bit, then double)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`+`, `-`, SUM and AVERAGE round the exact result to a 64-bit significand and then to a double, as Excel does.

## Context
T-1496 probed this: probes-numeric/double-rounding-ops A33-A64 (`a+b`, `SUM(a,b)`, `AVERAGE(a,b)` (sum rounded twice, then halved) and `a-b` with operands whose exponents differ by 12-52) are operand sets where single and double rounding differ, and Excel (excel-graph) returns the double-rounded value in every cell. The engine returns the singly-rounded value. Semantics: docs/excel-semantics/numbers.md "Division rounds twice". Helper to reuse or extend: `arithmetic::extended_quotient`. Code path: the addition/subtraction paths and the SUM/AVERAGE accumulators.

## Acceptance criteria
- [x] probes-numeric/double-rounding-ops A33-A64 pass; `rustsheet-eval show probes-numeric/double-rounding-ops.case.json` lists no failure in them, and no gated suite regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`arithmetic::extended_sum` forms the exact sum in 128 bits and rounds it to 64 bits, then 53; used by `+`, `-`, `SumParts::add` (SUM) and AVERAGE's accumulator. AVERAGE's final division stays the plain `/` (exact for the probed pair; division by larger counts is unprobed). probes-numeric/double-rounding-ops A33-A64 pass (41 -> 73 of 89 cells); no gated suite regressed. A65-A80 (SQRT, `^0.5`) remain.

Performance: a TwoSum pre-check (`sum_may_round_twice`) keeps exact sums and every sum whose rounding error is not within 2^-8 of half an ulp on the plain `+`; only near-midpoint sums take the 128-bit path (performance review measured the all-128-bit version at ~2.5x on a 200k-cell SUM/AVERAGE recalc).
