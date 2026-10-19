---
id: T-0440
title: "Defined-name LAMBDA calls share the parsed body instead of cloning it per call"
milestone: M3
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Calling a defined-name LAMBDA (`MyFn(x)`, or `MyFn` passed to a helper) no longer deep-clones its
parameters and body on every call, so `MAP(SEQUENCE(2^20), LAMBDA(x, MyFn(x)))` gets substantially
faster, with unchanged results.

## Context
T-0425 performance review (MINOR): model.rs (`NamedFunctionKind` and `DefinedNameKind` evaluation,
and `LambdaDefKind`) clones the whole `ParsedDefinedName::LambdaDefinition` into `Model::lambdas`
under a new id per call, and `lambda_definition` (functions/logical/lambda.rs) clones the body again
before patching it. For the benchmark above that is about 2M body clones per evaluation, and
`lambdas` grows by 1M entries until `clear_lambdas`, which likely explains why the 2nd and 3rd
evaluations of one model were slower than the 1st (T-0425 Outcome: 3.14s / 4.22s / 4.27s). Options:
hold LAMBDA bodies behind `Rc` in `ParsedDefinedName::LambdaDefinition` and `lambdas`, and/or hand
out one lambda id per defined name per evaluation.

## Acceptance criteria
- [x] Release time of the benchmark above, three evaluations of one model, before/after in the Outcome.
- [x] LAMBDA, LET and helper tests still pass; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Closure capture semantics (T-0439).

## Outcome
`ParsedDefinedName::LambdaDefinition` and `Model::lambdas` hold the parameters and body behind an
`Arc` (`SharedLambda`, functions/logical/lambda.rs; `Arc`, not `Rc`, like `ParsedDefinedName::Formula`),
so naming a defined-name LAMBDA copies no body.
The new `NamedLambdas` gives each defined-name LAMBDA one lambda id per evaluation
(`Model::defined_name_lambda`), so `lambdas` no longer grows by one entry per call. It also keeps
the body a call patched for the next call (`Model::call_lambda`), using T-0425's `call_prepared`
reuse rule; only a body is kept, not a wrong-count error. The body is taken out while a call runs, so recursion and `MyFn(MyFn(x))` patch their
own. `lambda_definition` checks the argument count before it copies the body. New test
`defined_name_lambda_calls_reuse_their_body_and_stay_independent` (recursion, nesting, a wrong
argument count, the name passed by value, a definition changed between evaluations).

Release, `=SUM(MAP(SEQUENCE(2^20), LAMBDA(x, MyFn(x))))` with `MyFn = LAMBDA(x, 2*x+1)`, three
evaluations of one model: before 2.59s / 4.56s / 4.03s, after 0.80s / 0.75s / 0.80s. The 2nd and
3rd evaluations are no longer slower than the 1st. Eval delta: none (perf only).
