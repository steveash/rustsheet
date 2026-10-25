---
id: T-2649
title: "ISOMITTED of an empty value (IFERROR of a blank, a blank array element) handed to a LAMBDA is FALSE"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A LAMBDA parameter bound to Excel's empty value (what IFERROR makes of a blank cell, a blank element of an HSTACK, VSTACK or
EXPAND array) is not omitted: ISOMITTED of it is FALSE, while a written empty argument `(1,)` stays omitted (TRUE).

## Context
Found by T-2461 (probes/t2461-blank-array-sources, excel-graph, 2026-10-25; arrays.md T-2413 paragraph):
- A12 `TEXTJOIN(",",FALSE,MAP(HSTACK(D!Z1:Z2,{1;2}),LAMBDA(v,ISOMITTED(v))))` is "FALSE,FALSE,FALSE,FALSE" (ours TRUE,FALSE,TRUE,FALSE);
- A13 `LAMBDA(x,ISOMITTED(x))(IFERROR(D!Z1,1))` FALSE (ours TRUE);
- A14 `LAMBDA(x,ISOMITTED(x))(INDEX(HSTACK(D!Z1:Z2,{1;2}),1,1))` FALSE (ours TRUE).
Ours represents the empty value as `CalcResult::EmptyArg`, the same value an omitted parameter is bound to, and
`fn_isomitted` (functions/information/isomitted.rs) tests for `EmptyArg` (it already special-cases LOOKUP's result). The
binding (functions/logical/lambda.rs, `lambda_cells` in bycol_byrow.rs) has to tell an omitted parameter from one bound to
the empty value, without changing how the value reads elsewhere (COUNT counts it, ISBLANK TRUE).

## Acceptance criteria
- [x] probes/t2461-blank-array-sources A12, A13, A14 leave eval/baseline/probes.txt, no new known failures.
- [x] arrays.md's ISOMITTED entries say "Ours" for the empty value.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Omission is now a property of how a parameter was bound, not of its value: `Binding::Omitted` (variable_stack.rs) is bound
for a missing argument and for an empty slot `(1,)`, reads as the empty value, and a LET name bound to it hands it on.
`fn_isomitted` reads the binding for a variable argument, so a parameter bound to the empty value (IFERROR of a blank, a
blank HSTACK element through MAP or INDEX) is FALSE. Eval: probes/t2461-blank-array-sources P!A12:A14 leave the baseline
(3 known failures fewer), no new failures.
