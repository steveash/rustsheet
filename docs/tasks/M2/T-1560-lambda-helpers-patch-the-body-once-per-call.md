---
id: T-1560
title: "LAMBDA helpers: patch the body once per call of MAP/SCAN/REDUCE/BYROW/BYCOL, not per element"
milestone: M2
lane: functions-logic
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
MAP, SCAN, REDUCE, BYROW and BYCOL clone the LAMBDA body and patch its variable ids once per call of the function, not once per element, with unchanged results.

## Context
Performance review of T-1549 (MINOR): `lambda_definition` clones the parameter Vec and the whole body AST on every element and `assign_variable_ids` walks the clone (functions/logical/lambda.rs, `call_lambda_with_values`). Over 2^20 elements that dominates the cost. Resolve and patch the body once per call, then rebind only the values per element, or share the body with `Arc`.

## Acceptance criteria
- [x] Per-element AST clone and id patching are gone from the five helpers; lambda unit tests and probes stay passing; `make check` green; reviewers approve.

## Out of scope
-

## Outcome
MAP, REDUCE, SCAN, BYROW, BYCOL and MAKEARRAY call their LAMBDA through a `LambdaCaller`
(functions/logical/lambda.rs): the definition is looked up and the body cloned and patched with
variable ids once, then each element only binds its values. A body that creates a LAMBDA (which holds
the parameter ids) makes the next call patch a fresh copy with fresh ids, as every call did before, so
closures behave as before (checked against the old code: `MAP(..,LAMBDA(x,LAMBDA(y,x+y)(10)))`,
REDUCE/SCAN with LAMBDA accumulators). BYROW/BYCOL over arrays (node arguments) use the same caller.
`call_lambda_with_values` is gone (no callers left), and `call_lambda` is a one-call `LambdaCaller`.
Tests cover a closure body in MAP, REDUCE, SCAN, BYROW, BYCOL and MAKEARRAY. `make check` (eval gate and
duplication ratchet included) green.
Measurement (release, one `evaluate` of `SUM(MAP(SEQUENCE(2^20),LAMBDA(x,IF(x>2,x*2+1,LET(t,x,t-1)))))`,
`REDUCE(0,SEQUENCE(2^20),LAMBDA(a,v,a+MOD(v,7)))` and `SUM(BYROW(SEQUENCE(262144,2),LAMBDA(r,SUM(r)*2)))`):
total for the three cells, two runs each: 2.13 s and 2.08 s before, 1.18 s and 1.41 s after, same
results. A body that creates any LAMBDA (a defined-name LAMBDA call too) still re-patches per element:
T-0425. Eval delta: none.
