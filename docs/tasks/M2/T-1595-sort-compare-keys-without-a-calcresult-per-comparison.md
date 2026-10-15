---
id: T-1595
title: "SORT: compare keys without a CalcResult per comparison and move rows instead of cloning"
milestone: M2
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SORT over an array of n rows costs about n log n cheap comparisons plus one pass over the rows, so
`REDUCE(0,SEQUENCE(3000),LAMBDA(a,x,SORT(VSTACK(a,x))))` is no longer dominated by SORT itself.

## Context
T-1593 timing (release, best of 5): the formula above takes about 2.15 s, against 29 ms for
`REDUCE(...,LAMBDA(a,x,VSTACK(a,x)))`, so almost all of it is SORT. `sort_key_cmp`
(functions/spill_functions.rs) converts both keys to a `CalcResult` (`array_node_to_calc_result`)
on every comparison before `compare_values`, and `fn_sort` clones each row of `data` (owned) into the
result instead of moving it. Keep the order exactly as now (the comparison must give the same
result for every pair, blanks last, errors as now); SORTBY may share the fix.

## Acceptance criteria
- [ ] Release timing in the Outcome, before and after, for the formula above.
- [ ] SORT and SORTBY results unchanged: their tests and eval cases pass, no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing SORT's ordering rules.
