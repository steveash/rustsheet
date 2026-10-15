---
id: T-1588
title: "LAMBDA parameters: share array values through the variable stack instead of cloning on each read"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A LAMBDA body that reads an array-valued parameter does not deep-copy the array on each read, so `REDUCE(0,SEQUENCE(n),LAMBDA(a,x,VSTACK(a,x)))` costs about one copy of the accumulator per step (VSTACK's own), not two.

## Context
Performance review of T-1561: after T-1561 moves REDUCE/SCAN's accumulator into the parameter, `VariableStack` (variable_stack.rs, `Binding::Value(value) => value.clone()`) still clones the bound value every time the body reads it, so a growing array accumulator stays O(n x |acc|) per REDUCE. Options: share arrays (`Rc`-backed array values or a borrow lent to the body), or move the value out on a parameter's last read. T-1561 measured 269 ms -> 147 ms for n = 3000; this task should measure the same formula before/after.

## Acceptance criteria
- [ ] Reading a parameter bound to an array does not deep-copy it; before/after timing of `=ROWS(REDUCE(0,SEQUENCE(3000),LAMBDA(a,x,VSTACK(a,x))))` (release) in the Outcome.
- [ ] Lambda unit tests and probes stay passing; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `CalcResult`'s size (SPEC-0010 constraints apply).
