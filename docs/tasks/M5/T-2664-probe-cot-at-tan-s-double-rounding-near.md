---
id: T-2664
title: "Probe COT at TAN's double-rounding near-ties (does COT inherit RN53(RN64(tan))?)"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence shows whether Excel's COT at TAN's 64-bit near-ties follows the double-rounded TAN that ours divides (COT = 1/TAN), and ours matches it or the gap is recorded.

## Context
T-2147 changed `excel_tan` (functions/math_and_trigonometry/trig.rs) to RN53(RN64(tan)), docs/excel-semantics/numbers.md
"TAN misses this model by 1 ulp". `fn_cot` (mathematical.rs) is `reciprocal_trig(excel_tan(x))`, so COT changed with it, but no COT
cell near a tie was probed. Use the excel-probe skill: a probes-numeric case with `=COT(A<n>)` at the 150 arguments of
probes-numeric/t2147-tan-double-rounding (copy its A column), filled with excel-graph.

## Acceptance criteria
- [ ] A probe case with oracle values; numbers.md records how many COT cells ours matches, and ours changes if a simple model fits better.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
