---
id: T-1544
title: "BETA.INV at a tiny p: Excel is inexact for a large alpha and floors near 1E-300; match its values"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
BETA.INV and BETAINV at a tiny p match Excel's values, including where Excel is not the exact root.

## Context
T-1543 stopped the `statrs` panic with a tail asymptotic and a bisection (`beta_quantile` in `functions/statistical/beta.rs`), which gives the exact root. Excel does not always: `BETA.INV(1E-200,10000,1)` = 0.8765595985431334 (exact 0.955), `BETA.INV(1E-300,2,5)` = 2.089E-154 (exact 2.58E-151, while 1E-200 gives the exact 2.58E-101), `BETA.INV(1E-300,0.5,0.5)` = `BETA.INV(1E-200,0.5,0.5)` = 2.7055792171147168E-151, `BETA.INV(1E-20,10000,1)` is `#NUM!`. Probe case: probes-numeric `t1543-beta-inv-tiny-p` (63 cells); statistics.md (BETA.INV at a tiny probability). Excel looks like an iteration that stops early; fit it (see T-0967 for the method).

## Acceptance criteria
- [ ] All 63 cells of `t1543-beta-inv-tiny-p` pass or the residual is a recorded DEVIATION.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
