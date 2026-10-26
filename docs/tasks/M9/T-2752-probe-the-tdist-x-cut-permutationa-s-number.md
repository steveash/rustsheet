---
id: T-2752
title: "Probe the TDIST x cut, PERMUTATIONA's number limit and POISSON's EXP overflow (T-2474 follow-up)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the three cuts T-2474 implemented as UNVERIFIED, so the engine's choices are either confirmed or
corrected by a later task.

## Context
T-2474 (docs/excel-semantics/statistics.md "Scalar distributions at extreme arguments, part 2 (T-2256)" and "POISSON at
x 0 with a negative mean") implemented:
- T.DIST (cumulative), T.DIST.RT, T.DIST.2T and TDIST are `#NUM!` where `x²` overflows (|x| ≥ about 1.34E+154), for
  either sign. Evidence only brackets it: `T.DIST.2T(1E+15,1)` is a number, 1E+300 is `#NUM!`. Negative x and the
  non-cumulative T.DIST at 1E+300 have no unmasked cell.
- PERMUTATIONA takes PERMUT's limit: a number of 2^31 - 1 or more is `#NUM!`. Evidence: 123456789.123 is a number,
  1E+15 is `#NUM!`.
- POISSON with x 0 and a mean below about -709.78 (EXP(-mean) overflows) is `#NUM!` in ours; unprobed.
Write one probe case with the `excel-probe` skill (eval/suites/probes/cases/), for example:
`T.DIST.2T(1E+150,1)`, `T.DIST.2T(1.3E+154,1)`, `T.DIST.2T(1.35E+154,1)`, `T.DIST.RT(1E+200,3)`,
`T.DIST(-1E+300,3,TRUE)`, `T.DIST.RT(-1E+300,3)`, `T.DIST(1E+300,3,FALSE)`, `TDIST(1E+100,3,1)`;
`PERMUTATIONA(2147483646,1)`, `PERMUTATIONA(2147483647,1)`, `PERMUTATIONA(2147483647,0)`, `PERMUTATIONA(1E+10,1)`,
`PERMUTATIONA(9.99E+14,1)`; `POISSON(0,-709,FALSE)`, `POISSON(0,-710,TRUE)`, `POISSON(0,-1E+300,FALSE)`.

## Acceptance criteria
- [ ] A probe case with oracle values (excel-graph) for the cells above; its failures recorded in eval/baseline by
      `make eval-baseline`, no other baseline line changed.
- [ ] statistics.md's two entries state the measured cuts (VERIFIED) and name a fix task for any cell ours gets wrong.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (a follow-up task if the probe disagrees).
