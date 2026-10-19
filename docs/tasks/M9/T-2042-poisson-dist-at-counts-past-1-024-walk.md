---
id: T-2042
title: "POISSON.DIST at counts past 1,024: walk the series from near the mode instead of falling back to statrs"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
POISSON.DIST (and POISSON) with a count past 1,024 match Excel within an ulp, as the smaller counts do since T-0927.

## Context
T-0927 (`functions/statistical/discrete.rs`) computes POISSON.DIST as `fl(fl(S)*exp(-mean))`, S the partial
exponential series walked term by term in double-double (docs/excel-semantics/statistics.md "Discrete distributions:
within an ulp of the exact sum"). A walk is capped at `MAX_STEPS` = 2^10 steps, so a count past that falls back to
statrs' incomplete gamma, about 1E-9 relative off: ironcalc-calc STATISTICAL/POISSON.xlsx D31
(`POISSON.DIST(1000000,1000000,TRUE)`: Excel 0.5002659614862837, ours 0.5002659618281267) and D55
(`POISSON.DIST(46040,46071,TRUE)`: Excel 0.4438020698095389, ours 0.44380206978608977) stay in the baseline. Both are
the exact value rounded (60-digit `decimal` sum), and an uncapped walk gives both (tried in T-0927 with 2^20 steps:
1,000,001 terms in about 25 ms, too slow for the lifting weight).
Approach to try: the terms that matter lie within about 40 standard deviations of the mean, so start the walk there
from an accurate pmf (Loader's saddle point, `stirlerr` and `bd0`, in double-double, or the walk's own ratio from a
scaled start), keeping the per-call bound near 2^10 steps; or a uniform asymptotic expansion (Temme) of Q(k+1, m).

## Acceptance criteria
- [ ] ironcalc-calc POISSON.xlsx D31 and D55 leave `eval/baseline/ironcalc-calc.txt`; no new lines.
- [ ] The per-call cost stays bounded (state it) and the lifting weight in lift/kinds/statistical.rs covers it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The one-ulp residue of POISSON.DIST D8, D10, D19 and D60 (Excel's own incomplete-gamma kernel, T-0971).
