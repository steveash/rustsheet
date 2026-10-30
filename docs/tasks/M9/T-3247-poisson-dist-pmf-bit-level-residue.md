---
id: T-3247
title: "POISSON.DIST pmf: bit-level residue of the saddle point (large k log form, small-k one ulp)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
POISSON.DIST's pmf is Excel's double where T-2055's model is still a few ulp off.

## Context
T-2055 (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms", `statistical/poisson.rs`) leaves:
- the saddle point at large k and a mean well off k: `POISSON.DIST(100,140,FALSE)` 23 ulp, `(300,240)` 46 ulp,
  `(300,210)` 16, `(30,21)` 11 (probes-numeric/t2055-poisson-pmf-sweep). Two suspects: `loader.rs` `excel_bd0`'s series
  (a fully stored series gets (300,210) and (300,240) but loses (300,420)), and the stored E = fl(-stirlerr - bd0)
  (Excel's binomial pmf holds its exponent extended, statistics.md "BINOM.DIST pmf");
- about one ulp at k = 2 to 8 (the per-k stirlerr of k = 2..6 is a fit, not a derived form);
- k = 1 and mean <= 1.1 cells one ulp off the closed form (`POISSON(1,0.01,FALSE)`, `POISSON.DIST(9,1E-10,FALSE)`).
Every such cell is within 4 ulp or within 15 digits except the large-k cells above.

## Acceptance criteria
- [ ] More bit-exact cells on t2055-poisson-pmf-sweep and -switch, and no Excel-computed pmf cell goes from pass to fail.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdf (T-3241).
