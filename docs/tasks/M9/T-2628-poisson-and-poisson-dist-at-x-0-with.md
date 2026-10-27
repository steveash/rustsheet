---
id: T-2628
title: "POISSON and POISSON.DIST at x 0 with a negative mean are EXP(-mean)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
POISSON and POISSON.DIST give Excel's EXP(-mean) at x 0 (floored) for a negative mean, cumulative or not, instead of `#NUM!`.

## Context
probes/poisson-negative-mean (excel-graph, 2026-10-24, T-2484): `POISSON(0,-1,2)`, `POISSON(0,-1,TRUE)`, `POISSON(0,-1,FALSE)`
and `POISSON.DIST(0,-1,TRUE)` are 2.718281828459045, `POISSON.DIST(0,-0.5,FALSE)` is 1.6487212707001282, while
`POISSON(1,-1,TRUE)` stays `#NUM!` (A1-A6). Ours is `#NUM!` for all of them. docs/excel-semantics/statistics.md
"POISSON at x 0 with a negative mean" records it. Unprobed: a fractional x below 1 (0.5), x 0 with mean -1E+300 (overflow),
and whether the mean check simply runs only for x >= 1; probe those before fitting.
Code: crates/rustsheet-engine/src/functions/statistical/poisson.rs.

Owned gen-m9 cell (T-2484 triage): gen-m9/23d82b62e9a85bdd Calc!A208
`=RECEIVED(39880,(ROUNDDOWN(0.285,(-2.5))/POISSON(0,(-1),Data!A9)),(RANK.AVG(Data!E11,Data!E3:E7)+AVERAGE(42)),(-0.5),2)`:
Excel `#N/A`, ours `#NUM!`; the maturity `0/POISSON(0,-1,2)` is 0 in Excel (probe A7), so the investment's RANK.AVG `#N/A`
comes next (RECEIVED reads every argument before the date order, probes/discount-argument-check-order).
Cells:
- gen-m9/23d82b62e9a85bdd.case.json (1): Calc!A208

## Acceptance criteria
- [ ] probes/poisson-negative-mean A1-A7 pass (`make eval-baseline` drops their lines).
- [ ] Visible parity (gen-m9): the known-failure line of gen-m9/23d82b62e9a85bdd Calc!A208 disappears from
      eval/baseline/gen-m9.txt, with no new known-failure lines in any baseline.
- [ ] Unit test for x 0 and x 1 with a negative mean.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other distributions' negative-parameter edges.

## Outcome
Done by T-2474 (scalar distributions at extreme arguments, part 2), which implements the same rule in poisson.rs:
probes/poisson-negative-mean A1-A7 and gen-m9/23d82b62e9a85bdd A208 pass, with unit tests for x 0, 0.3 and 1.
