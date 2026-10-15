---
id: T-1655
title: "Probe GCD/LCM/MULTINOMIAL: array-literal text or TRUE after a negative, error order, and 2^53 overflow against a later error"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The remaining GCD/LCM/MULTINOMIAL error-precedence paths are probed and matched.

## Context
T-1574 left unprobed (docs/excel-semantics/coercion.md, GCD / LCM / MULTINOMIAL range cells): array-literal text or TRUE after a negative for LCM and MULTINOMIAL; which of two different non-negative errors wins (e.g. `GCD("a",NA())`, `GCD(NA(),"a")`, and an omitted first argument against a later error); `GCD(1E+16,"a")` and `GCD(1E+16,NA())` (the engine returns the 2^53 `#NUM!` at once). Use the excel-probe skill, then match in gcd_lcm.rs and multinomial.rs.

## Acceptance criteria
- [ ] Probe cells with oracle values pass; coercion.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
