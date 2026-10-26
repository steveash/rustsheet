---
id: T-2804
title: "SPEC-0024 step 2: the aggregate walker charges the array cells it reads"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2803]
---

## Goal
A LAMBDA that reads a LET-bound array again and again is bounded by the formula work budget. For example,
`=LET(s,SEQUENCE(100000),REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(s))))` reads 1E10 cells and builds only 2E5.

## Context
SPEC-0024 rollout step 2, Design section 1 ("array cells read"). In aggregate_args.rs, `walk_array_items` (the array arm of
the aggregate walker behind SUM, COUNT, AVERAGE, MIN, MAX, PRODUCT and the other `for_each_value` readers) charges
`Model::spend_work` the cells of the array before it walks them. The budget and its error come from T-2803.

## Acceptance criteria
- [ ] `=LET(s,SEQUENCE(100000),REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(s))))` is `#VALUE!` within the harness timeout.
      An engine test asserts the value and the units spent (no timing).
- [ ] A counter test: `=SUM(SEQUENCE(10))` spends 10 more units than before this task.
- [ ] No baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other whole-array readers (T-2805).
