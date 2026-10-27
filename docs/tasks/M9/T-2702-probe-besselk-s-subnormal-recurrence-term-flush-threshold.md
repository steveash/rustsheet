---
id: T-2702
title: "Probe BESSELK's subnormal recurrence-term flush threshold between 1.34E-308 and 2.67E-308"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The cutoff below which Excel drops BESSELK's recurrence term `2n/x * K(n)` is known, not inferred.

## Context
T-2549 made `bessel_k` (functions/engineering/transcendental/bessel_k.rs) flush a term below the smallest normal double
(2.2250738585072014E-308, `arithmetic::flush_subnormal`). The evidence (probes-numeric/m9-besselk-underflow, T-2550) only
brackets the cutoff in (1.34E-308, 2.67E-308]: the x = 700 products of n = 1 (dropped) and n = 2 (kept). See
engineering.md "BESSELK near the bottom of the double range". Pick x between 700 and 702 so that the n = 1 or n = 2 product
falls at several points inside that bracket (compute the products from our K0/K1, which are bit-exact there), and ask
Excel for BESSELK(x,2) and (x,3) at each.

## Acceptance criteria
- [ ] A probes-numeric case (excel-graph) brackets the cutoff to within a few percent, and engineering.md records it.
- [ ] If the cutoff is not the smallest normal double, a functions-engineering task is filed to move it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BESSELI overflow cut (T-2613).
