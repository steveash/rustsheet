---
id: T-0185
title: "SUM of values that overflow the double range is #NUM!"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`SUM(9E+307,9E+307)` is `#NUM!` and stays an error through `&`, CONCAT and ISERROR, like the arithmetic operators.

## Context
Found by T-0173. Excel: probes/arith-overflow Calc!A4:D4 (`#NUM!`, ERROR.TYPE 6); the engine returns infinity. docs/excel-semantics/numbers.md "Overflow in `+ - * /` is #NUM! where it happens". Check other aggregate functions (SUMPRODUCT, AVERAGE, PRODUCT) against a probe before changing them.

## Acceptance criteria
- [ ] probes/arith-overflow Calc!B4, C4, D4 pass and leave eval/baseline/probes.txt.
- [ ] A unit test pins SUM overflow; behavior of other functions is only changed with oracle evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
