---
id: T-2706
title: "Probe subnormal companions for the unprobed density paths: BINOM.DIST k = n, cumulative tails, legacy names, second regimes"
milestone: M9
lane: research
status: done
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
- [x] A probes case holds each path above with its `(x)=0` companion, filled from excel-graph.
- [x] numbers.md records kept or flushed per path, citing the cells; a task is filed for each mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing engine code (the filed tasks do).

## Outcome
probes/t2706-subnormal-density-paths (35 calls with companions, excel-graph 2026-10-25) covers k = n, every cumulative
mode, the legacy names, a second regime per function and the statrs fallbacks (BETA.DIST past MAXGAM; the GAMMA.DIST
non-normal x / scale). Recorded in numbers.md "Distribution densities". The engine misses 12 rows: 11 for T-2737 (the flushes
and the NEGBINOM.DIST f = 0 keep), row 25 for T-2738 (GAMMA.DIST's #NUM! at a 1E+308 scale).
