---
id: T-2706
title: "Probe subnormal companions for the unprobed density paths: BINOM.DIST k = n, cumulative tails, legacy names, second regimes"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (`(x)=0` companions, decision 0042) settles whether a subnormal result is kept or flushed on the density
paths T-2703 left unprobed, and numbers.md "Distribution densities" records each answer.

## Context
T-2703 flushes the EXPON.DIST, WEIBULL, BETA.DIST and GAMMA.DIST densities and the BINOM.DIST (off k = 0 and k = n) and
NEGBINOM.DIST pmfs, from one probed regime each (numbers.md "Distribution densities"). Excel's flush follows each
function's internal path, not the size of the result, so these are unprobed:
- the BINOM.DIST pmf at k = n, Excel's EXP(fl(n·LN(p))) closed form (statistics.md "BINOM.DIST pmf: closed forms at k = 0
  and k = n"), left unflushed by T-2703 because EXP keeps a subnormal;
- every cumulative mode with a subnormal result (BINOM.DIST, NEGBINOM.DIST, BETA.DIST, GAMMA.DIST lower tails);
- the legacy names NEGBINOMDIST, GAMMADIST, EXPONDIST and the WEIBULL.DIST name (the WEIBULL evidence is the legacy name);
- a second argument regime per function, including the statrs fallbacks `dist.pdf`/`dist.pmf` in beta.rs, gamma.rs,
  weibull.rs and binom.rs.
Write a probes case with each formula and its companion (excel-probe skill); file an implementation task for any
mismatch.

## Acceptance criteria
- [ ] A probes case holds each path above with its `(x)=0` companion, filled from excel-graph.
- [ ] numbers.md records kept or flushed per path, citing the cells; a task is filed for each mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing engine code (the filed tasks do).
