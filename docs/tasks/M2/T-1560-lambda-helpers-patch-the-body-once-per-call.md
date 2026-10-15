---
id: T-1560
title: "LAMBDA helpers: patch the body once per call of MAP/SCAN/REDUCE/BYROW/BYCOL, not per element"
milestone: M2
lane: functions-logic
status: in-progress
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
- [ ] Per-element AST clone and id patching are gone from the five helpers; lambda unit tests and probes stay passing; `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
