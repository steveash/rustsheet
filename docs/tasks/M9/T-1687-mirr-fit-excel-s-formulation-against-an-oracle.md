---
id: T-1687
title: "MIRR: fit Excel's formulation against an oracle sweep"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MIRR matches Excel's last digits: its formulation is fitted against an oracle sweep and implemented.

## Context
Split out of T-0916, which implemented IRR and XIRR. Its acceptance named the MIRR F lines of
eval/baseline/ironcalc-calc.txt (MIRR.xlsx, 51 NumericUlp cells), e.g. `-MIRR(A2:B2,C2,D2)`: Excel
0.19999999999999996, engine 0.20000000000000007. probes-numeric/m9-fin-irr-rate has 35 MIRR cells, 32
of which pass. T-0916 suggested trying `(-NPV(rr, positives)*(1+rr)^n / (NPV(fr, negatives)*(1+fr)))^(1/(n-1)) - 1`
and its variants (financial.md "IRR, RATE and XIRR stop early"). Decision 0056 applies: a fit-lab fit
over a sweep, time-boxed.

## Acceptance criteria
- [x] A fit-lab fit (`mirr`) with its candidates' counts recorded; the winner implemented.
- [x] The MIRR lines leave eval/baseline/ironcalc-calc.txt with no new baseline lines.
- [x] financial.md records the formulation with evidence cells; `make check` green; reviewers approve.

## Out of scope
- RATE (not fitted; financial.md "RATE iterates on the rate itself").

## Outcome
New sweep probes-numeric/m9-fin-mirr (excel-graph, 432 cells: fixed families, 200 seeded random flows, and 6x6
rate grids on two- and three-flow families that isolate the reinvest path). Fit lab `mirr` (kernels/mirr.py),
467 cells with m9-fin-irr-rate's MIRR cells, bit-exact / within 4 ulp:

| candidate | exact | <=4 ulp |
|---|---|---|
| period-0 sums, `P*(1+rr)^n/(1+rr)`, Excel `^` (implemented) | 467 | 467 |
| same, `exp(ln q/(n-1))` | 456 | 467 |
| same, libm `pow` | 456 | 465 |
| period-0 sums, `P*(1+rr)^(n-1)`, Excel `^` / exp ln / libm | 413 / 410 / 406 | 451 / 451 / 450 |
| direct FV and PV, Excel `^` / libm / exp ln | 407 / 399 / 398 | 447 / 445 / 447 |
| Microsoft's NPV form (the old engine), Excel `^` / exp ln / libm | 365 / 362 / 361 | 437 / 437 / 436 |

`fn_mirr` now uses the winner (financial.md "MIRR's evaluation order"). Eval delta: ironcalc-calc fixed 55
(every MIRR.xlsx line; MIRR.xlsx 225/225), 0 regressions; probes-numeric (not gated) m9-fin-irr-rate MIRR 35/35
(was 32), m9-fin-mirr 432/432.
