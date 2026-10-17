---
id: T-1687
title: "MIRR: fit Excel's formulation against an oracle sweep"
milestone: M9
lane: functions-financial
status: in-progress
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
- [ ] A fit-lab fit (`mirr`) with its candidates' counts recorded; the winner implemented.
- [ ] The MIRR lines leave eval/baseline/ironcalc-calc.txt with no new baseline lines.
- [ ] financial.md records the formulation with evidence cells; `make check` green; reviewers approve.

## Out of scope
- RATE (not fitted; financial.md "RATE iterates on the rate itself").
