---
id: T-3150
title: "Incomplete beta at a shape of 1: probe Excel's w1 below 2^-53 (complement gives 0)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3007]
---

## Goal
Excel's value is known, and matched, where a0 = 1 outside the closed forms has an other tail w1 below 2^-53, which
`incomplete_beta::shape_one_tails`'s (0.5 - w) + 0.5 rounds to 0 or a multiple of 2^-53.

## Context
- T-3007 took w1 = (0.5 - w) + 0.5 at a0 = 1 with y0 > 0.9 from the oracle: every literal cell there agrees with it
  (statistics.md "Incomplete beta at a shape of 1 outside the closed forms"). None has w1 below about 1E-16, where the
  engine's earlier y0^b0 was accurate and the complement now gives 0 or a multiple of 2^-53. For example,
  `BETA.DIST(0.95,1000,1,TRUE)` was about 5.3E-23 and is now 0.
- One indirect hint that Excel agrees: `BETA.INV(1E-20,1000,1)` is `#NUM!` (probes-numeric/t1543-inverse-cdf-tiny-p A88).
- Probe it with the excel-probe skill: `BETA.DIST(0.95,1000,1,TRUE)`, `BETA.DIST(0.99,5000,1,TRUE)`,
  `F.DIST.RT(100,2,1000)`, `F.DIST(0.01,1000,2,TRUE)`, and a sweep of w1 from 1E-14 to 1E-300.

## Acceptance criteria
- [ ] A probe case with Excel's values for w1 from 1E-14 to 1E-300 at a0 = 1, y0 > 0.9.
- [ ] `shape_one_tails` matches them, or statistics.md records the finding (no gated cell regresses).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's w at a0 = 1 (T-3007).
