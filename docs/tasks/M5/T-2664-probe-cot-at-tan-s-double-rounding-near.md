---
id: T-2664
title: "Probe COT at TAN's double-rounding near-ties (does COT inherit RN53(RN64(tan))?)"
milestone: M5
lane: functions-math
status: done
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
- [x] A probe case with oracle values; numbers.md records how many COT cells ours matches, and ours changes if a simple model fits better.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes-numeric/t2664-cot-double-rounding (150 COT cells, excel-graph). Ours matches 131 of 150 (TAN: 129); the 19
misses (17 at 1 ulp, 2 at 2 ulp) are rows where TAN also misses, recorded in numbers.md. Simple alternative models fit far worse
(20/38 vs 36/38 at |x| <= 1.5), so the engine is unchanged. Eval delta: ungated suite probes-numeric, 131/150 bit-exact; no
baseline change, 0 regressions.
