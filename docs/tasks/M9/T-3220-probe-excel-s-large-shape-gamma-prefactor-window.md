---
id: T-3220
title: "Probe Excel's large-shape gamma prefactor window: near-mode bound, x below a/10, POISSON.DIST and large-shape BETA.DIST"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence pins where Excel's large-shape gamma prefactor takes its ratio form (T-2638) and where it does not,
so the engine's bracketed bounds (`incomplete_gamma::NEAR_MODE` = 0.4, `RATIO_FORM_MIN` = 0.02) and the functions that
use the form are set by evidence.

## Context
T-2638 found Excel's prefactor from shape 20 away from the mode: sqrt(a/2pi) exp(L + (a - x) - correction(a)) with L
the log of the ratio that is at least 1 (docs/excel-semantics/statistics.md, T-2638 paragraph). The evidence covers
GAMMA.DIST and CHISQ.DIST densities and cdfs at shapes 25, 50 and 500 for x from a/10 to 4a. Bracketed only:
- Near the mode: Excel is accurate at x = 0.99a (`POISSON.DIST(100,100,TRUE)`) and at 1E6; it takes the ratio form at
  1.5a and a/10; both agree at a/2. The engine switches at |x - a| = 0.4a (TOMS 654's bound, not evidence).
- Below a/10: the form is Excel's at a/21 (gen-m9/8dd2312c96effb1b A333); at a/500 Excel is neither form (probes/t2753-gamma-density-ratio-sweep A47, B47); the engine keeps
  `rcomp` below a/50. T-3219 owns those cells' fit.
- POISSON.DIST cumulative (`gamma_ur`) and the large-shape BETA.DIST / BETA.INV / F.INV paths keep `rcomp`: no cell
  probes the ratio form for them.
Probe (excel-probe skill): GAMMA.DIST(x,50,1,FALSE) and (x,200,1,FALSE) at x/a = 0.55 to 0.99 and 1.01 to 1.45 in
steps of 0.05; x/a = 0.002 to 0.05; POISSON.DIST(k,λ,TRUE) for k from 19 to 200 with λ/k from 0.1 to 3; BETA.DIST with
b in [20, 40) and a/b at most 1E-6.

## Acceptance criteria
- [ ] A probe case with oracle values lands, and statistics.md records the bounds it shows (or that the form does not
      apply), with the engine's constants and callers set to match (no regression).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The shapes below 20 (T-3219).
