---
id: T-1549
title: "MAP/SCAN/REDUCE: stop allocating an argument Vec and cloning the LAMBDA on every element"
milestone: M2
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MAP, SCAN and REDUCE over a large range allocate no argument `Vec` and clone no LAMBDA per element, with unchanged results.

## Context
Performance review of T-1465 (MINOR): `map_reduce.rs` and `scan.rs` build a `Vec<CalcResult>` of arguments and clone `lambda_result` for each element (2^20 small allocations over a full column). Reuse one buffer or let `call_lambda_with_values` take a slice and a reference to the lambda.

## Acceptance criteria
- [ ] Per-element argument Vec and LAMBDA clone are gone from MAP, SCAN and REDUCE; the lambda-helper probes and unit tests stay passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`call_lambda_with_values` takes the LAMBDA and its arguments by reference (`&CalcResult`, `&[CalcResult]`); MAP reuses one argument buffer, REDUCE, SCAN, BYROW/BYCOL and the (row, column) index loop pass a stack array and no longer allocate an argument Vec per element (the LAMBDA clone was only an integer id, so the gain is the Vec). `lambda_definition` borrows too. No behavior change; eval delta none; existing lambda unit tests and probes pass.
