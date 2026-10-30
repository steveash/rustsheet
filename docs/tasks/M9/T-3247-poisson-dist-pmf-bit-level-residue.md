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
campaign: visible
---

## Goal
POISSON.DIST's pmf is Excel's double where T-2055's model is still a few ulp off.

## Context
- The pmf switch is pinned only to [1.1, 1.100001) (T-2055). Probe means between them, which fit in 15 digits (1.10000000000001, 1.1000000001, 1.1000001), at k = 7, 8, 10 and 20 to place it (T-2055 fidelity review).
T-2055 (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms", `statistical/poisson.rs`) leaves:
- the saddle point at large k and a mean well off k: `POISSON.DIST(100,140,FALSE)` 23 ulp, `(300,240)` 46 ulp,
  `(300,210)` 16, `(30,21)` 11 (probes-numeric/t2055-poisson-pmf-sweep). Two suspects: `loader.rs` `excel_bd0`'s series
  (a fully stored series gets (300,210) and (300,240) but loses (300,420)), and the stored E = fl(-stirlerr - bd0)
  (Excel's binomial pmf holds its exponent extended, statistics.md "BINOM.DIST pmf");
- about one ulp at k = 2 to 8. `poisson.rs` `POISSON_STIRLERR_2_TO_5` (13, 12, 29, 34 ulp below the exact stirlerr at
  k = 2..5, marked `TODO(T-3247)`) is a fitted proxy, not a cause: the offsets are below the ulp of the stored exponent E,
  so they only flip E's rounding, and at every k some cell needs the other side. Find the real cause, most likely
  `excel_bd0`'s series arithmetic (unidentified for the beta density too, T-2586) or how E is stored or held, and drop
  the table. k = 7 and 8 also gain a cell or two at about -13 ulp, so the cause is probably not k-specific;
- the small-k edge of the closed form, 1 ulp off where main's statrs was right: `POISSON.DIST(1,0.3,FALSE)`, `(3,0.9)`,
  `(3,1.0)`, `(3,1.1)`, `(5,0.9)`, `(5,1.05)` (t2055-poisson-pmf-sweep A139, A153, A154, A156, A159, A161). At `(3,1.0)` and
  `(3,1.1)` the saddle point is Excel's double, so the mean <= 1.1 switch may not hold at every small k;
- k = 1 and mean <= 1.1 cells one ulp off the closed form (`POISSON(1,0.01,FALSE)`, `POISSON.DIST(9,1E-10,FALSE)`).
Every such cell is within 4 ulp or within 15 digits except the large-k cells above.

Gap triage 2026-10-30: the five large-k cells that are visibly off (t2055-poisson-pmf-sweep A193, A196, A197, A200,
A201) moved to T-3283 (`campaign: visible`, after T-3288's sweep), so the visible part is not parked here; this task keeps
the bit-level residue.

Stage: `campaign: visible` like T-2055 (Excel is the inaccurate side for this pmf, statistics.md), parked by
`defer: phase-1b` because only the four large-k cells are visibly off.

## Acceptance criteria
- [ ] `POISSON_STIRLERR_2_TO_5` is replaced by an identified mechanism, or the record says why none was found.
- [ ] More bit-exact cells on t2055-poisson-pmf-sweep and -switch, and no Excel-computed pmf cell goes from pass to fail.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdf (T-3269).
