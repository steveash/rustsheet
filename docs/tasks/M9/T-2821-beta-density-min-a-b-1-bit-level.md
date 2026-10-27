---
id: T-2821
title: "Beta density min(a,b) <= 1: bit-level residue after M2 (T-2503)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2503, T-2122, T-2512]
---

## Goal
BETA.DIST(x,a,b,FALSE) with min(a,b) <= 1 gives Excel's bits on the cells T-2503's model M2 leaves within a few ulp,
once Excel's GAMMALN (T-2122) and algdiv (T-2512) are reproduced, and the cells M2 itself leaves visibly off are
explained.

## Context
- T-2503 (`functions/statistical/beta_density.rs` `beta_density_small_shape`) reached the visible-parity target; its
  Outcome lists the probes-numeric cells that were bit-exact before and are now 1-4 ulp off: t2464-w2f-xruns-2 472
  (the whole (0.5, 0.7) block, +1 ulp; the engine's GAMMALN(1.2) is 1 ulp above Excel's -0.08537409000331583,
  probes-numeric t2513-r3-m1-residual-gl), t2513-r3-tdensity-a2 71, t2513-r3-m1-residual-hold 52, t0969-beta-switch 30,
  t2464-r2-tdist-p1 30, t2513-r3-m1-residual-a 25, t2513-verify-r3-m1-residual-hold 20, t0969-beta-density-b 9,
  t2513-r3-tdensity-hold 9, t2464-w2f-absweep 6, t2513-r3-fdensity-a 5, t0969-t-beta-link 2, t0969-beta-prefactor 1.
- Visibly off after T-2503 (each just past half a unit of the 15th digit):
  - the 4 old cells, all the algdiv branch at (0.0001, 50) and (50, 0.0001): probes-numeric m9-beta-dist-a0p0001 A44,
    A46 and m9-beta-dist-a50 A4, A6 (about 2E-15 relative; M2's own 4 misses in statistics.md);
  - newly: t2464-w2f-absweep A2642, A6668, A7054 (shapes next to (0.5, 6.25), the GAMMALN form) and
    t2513-r3-m1-residual-a D1152, 5-8 ulp.
- First step: score the same cells with Excel's measured GAMMALN at the shape arguments (research `m1lib.py`,
  `final_score.py`) to split the residue between T-2122 (GAMMALN), T-2512 (algdiv) and M2 itself.

## Acceptance criteria
- [ ] Each residue cell above is attributed to GAMMALN, algdiv or the M2 composition, with the attribution in
      docs/excel-semantics/statistics.md's "Beta density for min(a,b) <= 1" entry.
- [ ] Whatever is reachable once T-2122 / T-2512 land is bit-exact; no gated known-failure line is added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMALN's and algdiv's own kernels (T-2122, T-2512).
- min(a,b) > 1 (T-2507).
