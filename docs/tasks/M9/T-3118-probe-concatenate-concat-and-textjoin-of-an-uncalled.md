---
id: T-3118
title: "Probe CONCATENATE, CONCAT and TEXTJOIN of an uncalled LAMBDA, and TRIMRANGE's value read of a variable bound to one cell"
milestone: M9
lane: functions-text
status: todo
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
TRIMRANGE forms, fill it, then match it.

## Acceptance criteria
- [ ] The probe case passes (or its failures are owned by a filed task); each rule is in docs/excel-semantics/ with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
