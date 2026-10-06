---
id: T-0189
title: "Probe subnormal results of the % operator and of functions (EXP, SUM)"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for whether `=1E-306%`, `=EXP(-745)` and a SUM of tiny products flush a subnormal result to 0, as operators do (numbers.md "Subnormal results become 0").

## Context
Follow-up to T-0179 review. The engine flushes only `+ - * / ^` and POWER (`arithmetic::finite_or_num`); the postfix `%` (model.rs `OpUnary::Percentage`) and function results are not. Probe each with `&""` companions via the excel-probe skill; file an engine task for every cell the engine gets wrong.

## Acceptance criteria
- [x] A probe case with oracle sidecar covers `%`, EXP, SUM, PRODUCT and one more function.
- [x] numbers.md records the rule VERIFIED per function group; engine follow-ups are filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe subnormal-function-results (excel-graph) covers %, EXP, SUM, PRODUCT, SUMPRODUCT, SUMSQ, AVERAGE, SUBTOTAL, MIN, ABS, SIN and more, with C companion cells. Multiplicative results (%, PRODUCT, SUMPRODUCT, SUMSQ, AVERAGE) flush to a true 0; additive results (SUM, -) and EXP keep a subnormal that prints as 0. Recorded in numbers.md; engine tasks T-0987 (PRODUCT/SUMPRODUCT), T-0988 (postfix %), T-0989 (compare tiny value with 0). Literals below -308 exponent make the Graph workbook unsupported (noted). Eval delta: new-coverage failures only.
