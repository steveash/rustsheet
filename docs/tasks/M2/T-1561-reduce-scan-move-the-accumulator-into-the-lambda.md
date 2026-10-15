---
id: T-1561
title: "REDUCE/SCAN: move the accumulator into the LAMBDA instead of deep-copying it each step"
milestone: M2
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
REDUCE and SCAN move the accumulator into the LAMBDA each step instead of deep-copying it, with unchanged results.

## Context
Performance review of T-1549 (MINOR): `call_lambda_caller` (was `call_lambda_with_values`, T-1560) takes `&[CalcResult]` and clones each value into the variable stack, so with an array accumulator (`REDUCE(,r,LAMBDA(a,x,VSTACK(a,x)))`) each step copies the whole accumulator, O(n*|acc|). Add a by-value variant, or take `&mut [CalcResult]` and `std::mem::take` the values, for the REDUCE/SCAN path.

## Acceptance criteria
- [x] REDUCE and SCAN do not clone the accumulator per step; lambda unit tests and probes stay passing; `make check` green; reviewers approve.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`call_lambda_caller` (functions/logical/lambda.rs) takes its values by value
(`impl IntoIterator<Item = CalcResult>`) and moves each into its parameter, instead of cloning
from a slice. REDUCE and SCAN pass `[accumulator, element]`, so the accumulator is moved into
the LAMBDA each step; MAP drains its reused argument buffer; BYROW/BYCOL/MAKEARRAY pass arrays.
Reading the parameter in the body still copies it once per step (the variable stack hands out
clones); follow-up T-1588. SCAN also still clones the accumulator once per step for its output slot (the slot and the next step each need a value; cheap for the scalar accumulators SCAN's slots hold). Timing (release, `=ROWS(REDUCE(0,SEQUENCE(3000),
LAMBDA(a,x,VSTACK(a,x))))`, fastest of 5 recalcs): 269 ms before, 147 ms after. New test
`reduce_with_an_array_accumulator`. Eval delta: none.
