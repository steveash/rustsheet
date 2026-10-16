---
id: T-1655
title: "Probe GCD/LCM/MULTINOMIAL: array-literal text or TRUE after a negative, error order, and 2^53 overflow against a later error"
milestone: M5
lane: functions-math
status: done
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

## Outcome
New case `probes/gcd-lcm-multinomial-error-order` (52 cells, excel-graph): two different non-negative errors, omitted arguments against errors, an argument above 2^53 against errors and a negative, and array-literal text or TRUE next to a negative for LCM and MULTINOMIAL. Everything matched except GCD and LCM returning the 2^53 `#NUM!` at once; in Excel it loses to any other error in either order, like a negative's. Fixed in `gcd_lcm.rs`; MULTINOMIAL already matched. coercion.md updated. Eval delta: 0 fixed, 1 new case passing 52/52, 0 regressions.
