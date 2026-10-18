---
id: T-1919
title: "Probe a LAMBDA argument to lifted math functions (ABS, ROUND, MOD, CEILING.MATH), alone and beside an array"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel returns when a LAMBDA value is read as a number by a lifted math
function, so the engine's `#VALUE!` (T-1773) is confirmed or corrected.

## Context
T-1773 moved the math functions onto the SPEC-0011 driver; its review found a LAMBDA argument read
as `#N/IMPL!` there, and `Model::kernel_number` now maps it to `#VALUE!`, which is what the math
functions returned before and what MEDIAN and AND/OR/XOR return in the probes
(probes/aggregate-picked-direct-a-variants D7-D12, probes/and-or-xor-picked-text). No oracle cell
covers these functions. Test: `test_fn_round::a_lambda_number_argument_is_value_error`.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with oracle values for `ABS(LAMBDA(x,x))`,
      `ROUND(LAMBDA(x,x),1)`, `ROUND(1,LAMBDA(x,x))`, `MOD(7,LAMBDA(x,x))`,
      `CEILING.MATH(LAMBDA(x,x))`, `ROUND({1.25,2.5},LAMBDA(x,x))` (is it one error or a spilled
      array of errors?) and `ROUND(1/0,LAMBDA(x,x))` (which error wins).
- [ ] The finding is in docs/excel-semantics/ (lambda or coercion page); any mismatch with the
      engine is a filed task or fixed here if size S.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA arguments to non-math functions.
