---
id: T-2055
title: "POISSON.DIST: fit Excel's cdf and pmf (ironcalc-calc POISSON.xlsx and probes-numeric/t1048-poisson together)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
POISSON.DIST and POISSON match Excel on ironcalc-calc STATISTICAL/POISSON.xlsx without losing any excel-graph cell of
probes-numeric/t1048-poisson. The even-df CHISQ.DIST.RT identity of T-1778 must keep holding.

## Context
T-0927 tried two Poisson models; docs/excel-semantics/statistics.md "Discrete distributions: within an ulp of the exact
sum" has the detail.
- The exact value rounded matches 6 of 11 small ironcalc cdfs.
- `fl(fl(S)*exp(-mean))`, with S the partial exponential series, matches 8 of 11 small ironcalc cdfs and all 5 pmfs.
  It breaks 8 t1048-poisson cells that statrs gets right, for example `POISSON.DIST(10,4,FALSE)`.
- Routed through the identity `POISSON.DIST(k,m,TRUE)` = `CHISQ.DIST.RT(2m,2(k+1))`, that model also breaks
  m9-legacy-chidist A15/A16, where Excel's 1.0 became 1.0000000000000002.
POISSON.xlsx keeps 32 baseline lines: D31 and D55 have counts past 1,024, and the rest are near-ulp cells.
`functions/statistical/discrete.rs` has the double-double walk and the scaled exponent to reuse; the Poisson walk from
T-0927 is in that PR's history.
The likely root is Excel's incomplete gamma (T-0971, T-1778 rank 7).
Approach: fit against every Excel-computed POISSON, POISSON.DIST, CHISQ.DIST.RT and CHIDIST cell at once: ironcalc,
t1048-poisson, m9-legacy-chidist, m9-chisq-dist-rt and t1049-chisqdistrt. Compare cell by cell with main, ungated
suites included.

## Acceptance criteria
- [ ] Fewer POISSON.xlsx baseline lines; no Excel-computed cell in the suites above goes from pass to fail.
- [ ] The per-call cost stays bounded, and the lifting weight covers it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST and NEGBINOM.DIST (T-0927).
