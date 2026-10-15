---
id: T-1593
title: "LAMBDA parameters: move array values into more single-evaluation reads (IF branches, LET, SORT)"
milestone: M2
lane: functions-logic
status: done
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
- [x] Release timings in the Outcome, before and after, for the IF and SORT shapes above at n = 3000.
- [x] Each newly allowed position has a test where the parameter is also read twice (the copy still happens there).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `CalcResult`'s size.

## Outcome
`read_once` (functions/logical/lambda.rs) now counts, per parameter, how many times one evaluation
of the body reads it (0, 1 or "many"), walking the tree without recursion (parents first, then
counted children first). A read counts once when every step from the body is a group or an argument
that `evaluates_once` lists: VSTACK and HSTACK (as before), every argument of LET (each value bound
once, then the body) and of IF, and the array argument of SORT, TAKE and DROP. Each was audited, and
each implementation now says that `read_once` relies on it. The branches of an IF are exclusive, so
they count as the most any one of them reads: `IF(x>0,VSTACK(a,x),a)` reads `a` once. IF over an
array condition evaluates both branches, so `if_array` first turns every read-once binding into a
copied one (`VariableStack::copy_read_once`). Any other read still copies.
Tests: `reads_through_once_positions_count_once` and `reads_more_than_once_or_elsewhere_do_not_count_once`
(unit tests of the analysis, each position also read twice), and
`a_parameter_read_once_through_if_let_sort_take_drop_is_moved_and_one_read_twice_is_copied`
(end to end, including an array-condition IF that fails with `#ERROR!` without the copy).
Timing (release, `=ROWS(REDUCE(0,SEQUENCE(3000),LAMBDA(a,x,<body>)))`, best of 5 recalcs, before → after):
- `IF(x>0,VSTACK(a,x),a)`: 158.0 ms → 28.7 ms
- `SORT(VSTACK(a,x))`: 2288.9 ms → 2151.6 ms. The copy is gone, but SORT's own work dominates:
  it converts both keys to a `CalcResult` on every comparison and clones each row. Filed **T-1595** (perf).
- `LET(y,x*2,VSTACK(a,y))`: 156.7 ms → 31.6 ms
- `VSTACK(a,x)` for reference: 31.2 ms → 29.4 ms
Eval delta: none (0 regressions, 0 fixes in every gated suite).
