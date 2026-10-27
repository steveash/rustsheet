---
id: T-2737
title: "BINOM.DIST, NEGBINOM.DIST, GAMMA.DIST, EXPON.DIST cdfs and the CHISQ.DIST density flush a subnormal; the NEGBINOM.DIST pmf at f = 0 keeps it"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-2706]
---

## Goal
The engine returns Excel's kept-or-flushed subnormal on every path probes/t2706-subnormal-density-paths found it wrong
(numbers.md "Distribution densities", T-2706), so probes/t2706-subnormal-density-paths
fails only at A25:B25 (T-2738).

## Context
T-2706 probed each path with its `(x)=0` companion (decision 0042), excel-graph 2026-10-25. Excel flushes, the engine keeps:
- the BINOM.DIST cdf off k = 0 (A5:B7: `BINOM.DIST(1,1000,0.52,TRUE)`, `BINOM.DIST(3,1000,0.53,TRUE)`, BINOMDIST),
  binom.rs `binomial_cdf` / the incomplete-beta upper tail;
- the NEGBINOM.DIST cdf off f = 0 (A9:B9 `NEGBINOM.DIST(1,1000,0.48,TRUE)`);
- the GAMMA.DIST cdf (A19:B20 `GAMMA.DIST(1E-155,2,1,TRUE)`, GAMMADIST), gamma.rs `gamma_dist_cdf`;
- the EXPON.DIST cdf (A29:B29 `EXPON.DIST(1E-150,1E-160,TRUE)`), exponential.rs, while WEIBULL.DIST's cdf of the
  same 1E-310 is kept (A34:B34), so the two cannot share one form;
- the CHISQ.DIST density (A35:B35 `CHISQ.DIST(1430,2,FALSE)`);
- the BETA.DIST cdf at shapes (10, 2) (A15:B15 `BETA.DIST(1E-31,10,2,TRUE)`), while shapes (2, 1) keep it (A14, A16,
  BETADIST too): probe two or three shapes between before choosing a rule (excel-probe skill), or split BETA.DIST off.
Excel keeps, the engine flushes: the NEGBINOM.DIST pmf at f = 0, p^s (A12:B13, NEGBINOMDIST too), like the BINOM.DIST
pmf at k = n, an EXP-based closed form (binom.rs `fn_negbinom_dist`).

## Acceptance criteria
- [x] Each cell above matches; probes/t2706-subnormal-density-paths fails only A25:B25 (baseline shrinks).
- [x] numbers.md "Distribution densities" says which paths the engine flushes, citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- GAMMA.DIST's #NUM! at a 1E+308 scale (T-2738).

## Outcome
probes/t2737-subnormal-cdf-shapes (17 calls with companions) settles BETA.DIST: its cdf keeps a subnormal only at
beta = 1. The engine flushes the BINOM.DIST and NEGBINOM.DIST cdfs off their closed forms, the GAMMA.DIST, EXPON.DIST
and CHISQ.DIST cdfs, the CHISQ.DIST density and the BETA.DIST cdf off beta = 1, and keeps the NEGBINOM.DIST pmf at
f = 0; both probes pass but for t2706 row 25 (T-2738). Recorded in numbers.md "Distribution densities".
