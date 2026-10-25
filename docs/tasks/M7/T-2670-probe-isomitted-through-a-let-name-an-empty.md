---
id: T-2670
title: "Probe ISOMITTED through a LET name, an empty LET value and IF/CHOOSE passing an omitted parameter"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for ISOMITTED of forms ours decides without it: `LAMBDA(x,[y],LET(z,y,ISOMITTED(z)))(1)`,
`LET(z,,ISOMITTED(z))`, `LET(z,IFERROR(Z1,1),ISOMITTED(z))`, `LAMBDA(x,[y],LAMBDA(z,ISOMITTED(z))(y))(1)`, `LAMBDA(x,[y],ISOMITTED(IF(TRUE,y)))(1)`, `LAMBDA(x,[y],ISOMITTED(CHOOSE(1,y)))(1)`,
`LAMBDA(x,[y],ISOMITTED(IFERROR(y,1)))(1)`, `REDUCE(,{1,2},LAMBDA(a,v,ISOMITTED(a)))` and its SCAN twin (FALSE in ours since T-2649), and `ISOMITTED(IFERROR(Z1,1))` outside a LAMBDA (Z1 blank).

## Context
T-2649 made omission a property of the binding (`Binding::Omitted`, variable_stack.rs): a LET name bound to an omitted
parameter or to an empty slot hands the omission on (TRUE), and a non-variable argument is still judged by its value
(`EmptyArg` is TRUE except for LOOKUP), so the forms above are TRUE in ours with no evidence
(arrays.md, the T-2413/T-2649 paragraph and the "ISOMITTED of a function result" entry). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with these forms (and their FALSE controls) is oracle-filled; arrays.md records the results.
- [ ] Any mismatch is filed as a functions-logic task (or fixed here if it is one line in `fn_isomitted`/`bind_argument`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
