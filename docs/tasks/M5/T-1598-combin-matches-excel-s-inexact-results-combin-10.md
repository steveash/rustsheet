---
id: T-1598
title: "COMBIN matches Excel's inexact results (COMBIN(10,4) = 209.99999999999997)"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
COMBIN returns Excel's exact doubles, including its inexact ones.

## Context
T-1190's sweep (probes/multinomial-algorithm-sweep B1:B10, excel-graph 2026-10-15) shows Excel's COMBIN is not the
exact binomial: `COMBIN(10,4)` = 209.99999999999997, `COMBIN(40,20)` 137846528819.99997, `COMBIN(170,85)`
9.144841845131565e+49 (7 of 10 cells fail; we return the exact value). Its rounding differs from MULTINOMIAL's
(`MULTINOMIAL(4,6)` = 209.99999999999977), whose fit (log10 running sum, then `exp(x * ln 10)`;
docs/excel-semantics/numbers.md "MULTINOMIAL is inexact") is a first candidate family. Numeric fitting:
probe more (n up to 1029 where COMBIN stays finite, small k and k near n/2), then compare named candidates
(decision 0056). Semantics: numbers.md "COMBIN is inexact". Code: `fn_combin` (functions/math_and_trigonometry).

## Acceptance criteria
- [ ] probes/multinomial-algorithm-sweep B1:B10 pass, plus a new probe of 30+ argument pairs; candidates and scores recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COMBINA and MULTINOMIAL.
