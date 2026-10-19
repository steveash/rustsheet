---
id: T-0440
title: "Defined-name LAMBDA calls share the parsed body instead of cloning it per call"
milestone: M3
lane: perf
status: in-progress
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
- [ ] Release time of the benchmark above, three evaluations of one model, before/after in the Outcome.
- [ ] LAMBDA, LET and helper tests still pass; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Closure capture semantics (T-0439).
