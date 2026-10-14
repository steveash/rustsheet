---
id: T-1561
title: "REDUCE/SCAN: move the accumulator into the LAMBDA instead of deep-copying it each step"
milestone: M2
lane: functions-logic
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
REDUCE and SCAN move the accumulator into the LAMBDA each step instead of deep-copying it, with unchanged results.

## Context
Performance review of T-1549 (MINOR): `call_lambda_with_values` takes `&[CalcResult]` and clones each value into the variable stack, so with an array accumulator (`REDUCE(,r,LAMBDA(a,x,VSTACK(a,x)))`) each step copies the whole accumulator, O(n*|acc|). Add a by-value variant, or take `&mut [CalcResult]` and `std::mem::take` the values, for the REDUCE/SCAN path.

## Acceptance criteria
- [ ] REDUCE and SCAN do not clone the accumulator per step; lambda unit tests and probes stay passing; `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
