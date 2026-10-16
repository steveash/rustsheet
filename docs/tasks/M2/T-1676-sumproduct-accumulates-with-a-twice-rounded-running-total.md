---
id: T-1676
title: "SUMPRODUCT accumulates with a twice-rounded running total (T-1665 probe)"
milestone: M2
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMPRODUCT of a range sums with the same twice-rounded running total as SUM, so probes-numeric/double-rounding-avedev-deviations passes in full.

## Context
Found by T-1665: the control cells `Calc!A2, A5, A8, A11, A14, A17, A20, A23` (`=SUMPRODUCT(ABS(Z!A..:A..))` over the symmetric D, -D, e, -e operand sets) differ from Excel by one ulp in the engine, while AVEDEV (now `extended_sum`) matches Excel on the same deviations. Excel double-rounds that sum. Find SUMPRODUCT in functions/ and switch its accumulation to `extended_sum` (arithmetic.rs). Check SUMPRODUCT of products too (`a*b` terms) against numbers.md before changing.

## Acceptance criteria
- [x] The 8 SUMPRODUCT control cells of probes-numeric/double-rounding-avedev-deviations pass; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_sumproduct` adds each product to the running total with `arithmetic::extended_sum`; products stay plain (no evidence for twice-rounded multi-array products). probes-numeric/double-rounding-avedev-deviations passes 56/56 (the 8 control cells were the misses); no baseline entries existed for it, so no baseline change.
