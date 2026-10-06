---
id: T-0110
title: "Near-zero snapping in final addition/subtraction"
milestone: M1
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-0009, T-1017]
---

## Goal
Implement Excel's rule once VERIFIED in docs/excel-semantics/numbers.md.

## Context
Probe cells A2–A4, A15.

## Acceptance criteria
- [ ] Probe cells pass; rule documented with evidence; implemented in one place.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
The rule is now fitted (docs/excel-semantics/numbers.md): snap when the two addends have opposite
signs and the result is below 8 ULP of the smaller addend. This branch implements it at the
formula root. But Excel does **not** snap `=(X-b)`, and our AST and stored formula text drop the
outer parentheses. Shipping now is net +233 cells (347 fixed) but breaks the 114 `parenthesized-sub` cells
that pass today. Waiting for T-1017 (keep the outer parentheses), then skip grouped roots in
`compute_formula` and re-run `make eval`. Unblock when T-1017 is `done`.
