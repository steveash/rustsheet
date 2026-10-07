---
id: T-0110
title: "Near-zero snapping in final addition/subtraction"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0009]
---

## Goal
Implement Excel's rule once VERIFIED in docs/excel-semantics/numbers.md.

## Context
Probe cells A2–A4, A15. Also probes/literal-digits C4 (`=0.1+0.2-0.3` = 0, engine 5.55E-17), handed over by T-0144.

## Acceptance criteria
- [ ] Probe cells pass (including probes/literal-digits C4); rule documented with evidence; implemented in one place.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
