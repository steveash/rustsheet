---
id: T-0425
title: "LAMBDA helpers: keep the patched body when the LAMBDAs a body creates hold none of its parameter ids"
milestone: M3
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1560]
---

## Goal
MAP, REDUCE, SCAN, BYROW, BYCOL and MAKEARRAY keep their once-patched LAMBDA body (T-1560) when the
body calls a defined-name LAMBDA, an inline `LAMBDA(..)(..)` or an eta-reduced function, so
`MAP(r, LAMBDA(x, MyFn(x)))` gets the same speedup as a plain body, with unchanged results.

## Context
T-1560 performance review (MINOR): `call_prepared` (functions/logical/lambda.rs) re-patches the body
whenever `last_lambda_id` moved during the previous element, because a LAMBDA created by the body may
hold the parameter ids (`LAMBDA(x, LAMBDA(y, x+y))`). Any LAMBDA counts, so a defined-name LAMBDA call
(model.rs hands out a new lambda id per call) falls back to the per-element clone. Invalidate only when a
LAMBDA created during the call refers to one of the prepared ids (or capture values at creation).

## Acceptance criteria
- [x] `MAP(SEQUENCE(2^20), LAMBDA(x, MyFn(x)))` with a defined-name LAMBDA: release time before/after in the Outcome.
- [x] Closure tests in test_fn_map_reduce.rs still pass; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing how LAMBDA closures capture variables in general.

## Outcome
`call_prepared` (functions/logical/lambda.rs) now keeps its patched body unless a LAMBDA created
since the last check reads one of the prepared parameter ids (`lambdas_since_read`, over the new
explicit-stack `Node::reads_variable`). Defined-name LAMBDAs, eta-reduced functions and LAMBDAs made
inside a called one no longer force a per-element re-clone; a body that makes a LAMBDA over its own
parameters still re-patches every call, as before. Results are unchanged: new test
`helpers_calling_defined_name_and_eta_lambdas_see_each_element` plus the existing closure tests.
The patch reuse itself is shown by the benchmark, not asserted by a test.

Release, `=SUM(MAP(SEQUENCE(2^20), LAMBDA(x, MyFn(x))))` with `MyFn = LAMBDA(x, 2*x+1)`, three
evaluations of one model: before 3.37s / 4.64s / 4.90s, after 3.14s / 4.22s / 4.27s (−7% to −13%).
What remains is the defined-name body cloned twice per call: follow-up T-0440. Escaping closures
(`#NAME?` before and after this change): follow-up T-0439. Eval delta: none (perf only).
