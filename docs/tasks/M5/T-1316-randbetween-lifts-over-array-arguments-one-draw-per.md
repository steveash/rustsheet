---
id: T-1316
title: "RANDBETWEEN lifts over array arguments (one draw per element)"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
RANDBETWEEN with an array `bottom` or `top` returns an array of the broadcast shape, one draw per
element, as Excel does; today the engine returns `#N/IMPL!`.

## Context
probes/volatile-invariant Calc!F1:F9 (T-1283): `ROWS(RANDBETWEEN(SEQUENCE(1000,1,5.5,0),5.7))` is
1000 on excel-graph, and MIN/MAX over such a call are the run-invariant values in numbers.md
"RANDBETWEEN and RANDARRAY argument checks". The engine gives `#N/IMPL!` for all nine.
F1:F4 and F6:F9 also need T-1318's range rule (draw from CEILING(bottom) to FLOOR(top), never
below CEILING(bottom)); F5 needs only the lift. Code:
crates/rustsheet-engine/src/functions/math_and_trigonometry/random.rs; reuse the existing
array-lift helper other math functions use (search before writing one).

## Acceptance criteria
- [ ] Calc!F5 passes; F1:F4 and F6:F9 pass once T-1318 is also in; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The RNG and the distribution.
