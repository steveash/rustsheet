---
id: T-3241
title: "POISSON cdf: fit Excel's deep upper tail (mean well above k)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
POISSON and POISSON.DIST's cdf match Excel in the deep upper tail, where the mean is well above k (Q(k + 1, mean) with mean >= about 4(k + 1)), without losing any Excel-computed cell of CHISQ.DIST.RT, CHIDIST or GAMMA.DIST that shares the kernel.

## Context
T-2055 fitted the pmf (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms") and left the cdf on
`statistical/incomplete_gamma` `gamma_ur`. In the deep upper tail Excel is up to 60 ulp from the exact value and from
ours, with no shared pattern: `POISSON(2,100,TRUE)` = 1.8976107553682424E-40 (gen-m9 c7664e494e418395 A294; ours -35
ulp), `POISSON(3,100,TRUE)` (+60), `POISSON(6,100,TRUE)` (583ca2baea0bb30a A399, -34), `POISSON(9,100,TRUE)` (+44),
`POISSON(6,180,TRUE)` (+24), `POISSON(3,42,TRUE)` (+10). Not the pmf times its finite series 1 + k/m + ...: with Excel's own
pmf that is 45 ulp off at (2, 100). Not exp(fl(a LN x - x - GAMMALN(a))) times the series in any of 48 storage orders
tried (T-2055 scratch fit). This is the shared incomplete-gamma kernel's Q prefactor; coordinate with T-2638 and T-3060.

## Acceptance criteria
- [ ] The POISSON cdf cells above are within 15 digits of Excel, and no Excel-computed cell of POISSON, POISSON.DIST,
      CHISQ.DIST.RT, CHIDIST, GAMMA.DIST or GAMMADIST goes from pass to fail (main-vs-HEAD per cell).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The pmf (T-3242).
