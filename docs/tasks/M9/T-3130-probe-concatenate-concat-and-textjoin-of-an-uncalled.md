---
id: T-3130
title: "Probe CONCATENATE, CONCAT and TEXTJOIN of an uncalled LAMBDA, and TRIMRANGE's value read of a variable bound to one cell"
milestone: M9
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CONCATENATE, CONCAT and TEXTJOIN of an uncalled LAMBDA, and TRIMRANGE's value read of a LET variable bound to one cell, match Excel, with probe evidence.

## Context
T-2622 found that `LAMBDA(x,x)&""` is `#VALUE!` (probes/t2622-take-let-reference-lambda-text Q1-Q10) and fixed only the `&` operator
(`handle_concatenate`, arithmetic.rs); the text functions read their arguments through `get_string_or_array` (cast.rs), which still gives
`#N/IMPL!` for a LAMBDA (arrays.md "An uncalled LAMBDA is one value": unprobed). T-2622 also made TAKE, DROP and TRIMRANGE keep a variable
bound to one cell a reference as a reference argument, and TAKE/DROP in value context (lookup.md "TAKE/DROP of a variable bound to one cell
is a reference"); TRIMRANGE's value read (`TYPE(TRIMRANGE(LET(v,A2,v)))`, `ROWS(A2-TRIMRANGE(LET(v,A3,v)))` with A3 text) is unprobed.
Write a probe case (excel-probe skill) with CONCATENATE/CONCAT/TEXTJOIN of `LAMBDA(x,x)` and of a LET-bound one (ERROR.TYPE too), and the
TRIMRANGE forms, fill it, then match it. Also probe the `&` edges ours answers by its existing error order: `LAMBDA(x,x)&{1,2}`,
`{1,2}&LAMBDA(x,x)` (ours: an array of `#VALUE!`), `LAMBDA(x,x)&NA()` (ours `#VALUE!`) and `NA()&LAMBDA(x,x)` (ours `#N/A`); and the
cell-variable rule's unprobed forms: a variable as the arm CHOOSE picks (`LET(v,A2,TYPE(TAKE(CHOOSE(1,v),1)))`) and one bound to a cell on
another sheet (`TYPE(TAKE(LET(v,Data!A1,v),1))`).

## Acceptance criteria
- [ ] The probe case passes (or its failures are owned by a filed task); each rule is in docs/excel-semantics/ with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe t3130-concat-lambda-trimrange-cell-variable (28 cells) passes against excel-graph. CONCAT/TEXTJOIN of an uncalled LAMBDA are now #VALUE! (were #N/IMPL!); TRIMRANGE reads its source as a reference like TAKE, so a one-cell trim is a scalar. Rules in arrays.md and lookup.md.
