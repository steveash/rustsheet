---
id: T-0425
title: "LAMBDA helpers: keep the patched body when the LAMBDAs a body creates hold none of its parameter ids"
milestone: M3
lane: perf
status: todo
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
- [ ] `MAP(SEQUENCE(2^20), LAMBDA(x, MyFn(x)))` with a defined-name LAMBDA: release time before/after in the Outcome.
- [ ] Closure tests in test_fn_map_reduce.rs still pass; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing how LAMBDA closures capture variables in general.
