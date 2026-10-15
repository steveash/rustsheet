---
id: T-1593
title: "LAMBDA parameters: move array values into more single-evaluation reads (IF branches, LET, SORT)"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
More LAMBDA accumulator shapes avoid a deep copy of an array parameter per call, such as
`REDUCE(0,SEQUENCE(n),LAMBDA(a,x,IF(x>0,VSTACK(a,x),a))))`, a LET-wrapped body and `SORT(VSTACK(a,x))`.

## Context
T-1588 moves a parameter's value into its read only when the body reads it once, and that read
is the body itself or a direct VSTACK/HSTACK argument (`read_once`, functions/logical/lambda.rs;
`Binding::ReadOnce`, variable_stack.rs). Every other read still clones it. Options:
- Allow more single-evaluation positions after auditing that each function evaluates the argument
  once: LET's value and body arguments, the chosen IF branch (reads in exclusive branches count once
  each), and SORT, TAKE and DROP.
- Shared (`Rc`) array values. SPEC-0010's `CalcResult` constraints apply.

## Acceptance criteria
- [ ] Release timings in the Outcome, before and after, for the IF and SORT shapes above at n = 3000.
- [ ] Each newly allowed position has a test where the parameter is also read twice (the copy still happens there).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `CalcResult`'s size.
