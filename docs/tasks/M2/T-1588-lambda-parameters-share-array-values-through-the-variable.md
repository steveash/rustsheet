---
id: T-1588
title: "LAMBDA parameters: share array values through the variable stack instead of cloning on each read"
milestone: M2
lane: functions-logic
status: done
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
- [x] Reading a parameter bound to an array does not deep-copy it; before/after timing of `=ROWS(REDUCE(0,SEQUENCE(3000),LAMBDA(a,x,VSTACK(a,x))))` (release) in the Outcome.
- [x] Lambda unit tests and probes stay passing; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `CalcResult`'s size (SPEC-0010 constraints apply).

## Outcome
A LAMBDA parameter that the body reads exactly once per call is bound as `Binding::ReadOnce`
(variable_stack.rs), and that read moves the value out (`VariableStack::read_value`) instead of
cloning it. "Exactly once" is decided when the body is prepared (`read_once`, functions/logical/lambda.rs):
one read of the parameter in the whole tree, and that read is the body itself or a direct argument of
VSTACK or HSTACK (through groups), which evaluate each argument once. Every other read still copies,
since a function may evaluate an argument twice or a nested LAMBDA may read the parameter on each call.
A taken slot holds an error, so a second read would show rather than read 0. One walk
serves all parameters. The walk reuses `Node::push_children`, split out of `contains_function`.
Limit: other shapes still copy on each read, such as `IF(x>0,VSTACK(a,x),a)` (two reads), a
LET-wrapped body and `SORT(VSTACK(a,x))`. Follow-up T-1593. Duplicated lines went from 6369 to
6336 (`quality/duplication.json` updated by the script). Timing (release,
`=ROWS(REDUCE(0,SEQUENCE(3000),LAMBDA(a,x,VSTACK(a,x))))`, best of 5 recalcs): 147.1 ms before,
29.8 ms after. New test `a_parameter_read_once_is_moved_and_one_read_more_often_is_copied`.
Eval delta: none.
