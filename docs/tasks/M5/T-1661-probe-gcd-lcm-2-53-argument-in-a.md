---
id: T-1661
title: "Probe GCD/LCM: 2^53 argument in a range cell or array literal against a later error, and LCM result overflow against a later error"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The deferral of GCD/LCM's 2^53 `#NUM!` and LCM's result-overflow `#NUM!` is probed for range cells and array-literal elements, and the engine matches.

## Context
T-1655 probed typed scalars only (probes/gcd-lcm-multinomial-error-order); the engine defers the above-2^53 `#NUM!` for range cells and array elements too (`GCD(A1:A2)` with 1E16 and "a", `GCD({10^16,"a"})`), unconfirmed, and returns LCM's "result too large" at once (`LCM(2^26+1,2^27+1,"a")`-style). Add cells, fill with excel-probe, match in gcd_lcm.rs and multinomial.rs, update coercion.md.

## Acceptance criteria
- [ ] Probe cells with oracle values pass; coercion.md drops the T-1661 'inferred' note.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
