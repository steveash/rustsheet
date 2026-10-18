---
id: T-1598
title: "COMBIN matches Excel's inexact results (COMBIN(10,4) = 209.99999999999997)"
milestone: M5
lane: functions-math
status: done
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
- [x] probes/multinomial-algorithm-sweep B1:B10 pass, plus a new probe of 30+ argument pairs; candidates and scores recorded.
- [x] `make check` green; reviewers approve.

## Out of scope
- COMBINA and MULTINOMIAL.

## Outcome
Fitted and implemented (2026-10-17). New probe probes/combin-algorithm-sweep (844 cells: every k for n = 1..32, plus small,
n/3, n/2 and near-n k for n = 40..1030; excel-graph). Excel's COMBIN, with `j = min(k, n-k)`, multiplies the rounded
quotients `(n-j+i)/(i+1)` for i = 1..j-1 into one double product and then multiplies by `n`: 841/841 numeric cells
bit-exact. Fit-lab style scores (out of 841): correctly rounded 572; textbook `r*(n-j+i)/i` 569; best other natural
pairing/order 570; shifted pairing with `n` first 624, over k instead of min 636, unfused quotient 569; log-domain sums at
most 6/87 of the j = 3 cells. `combin` (mathematical.rs) replaces the old k-step loop, so a huge k now overflows to
`#NUM!` within a few hundred factors instead of looping k times. Eval: probes 3004 → 2997 known failures
(multinomial-algorithm-sweep B2:B5, B7:B9), ironcalc-calc 1300 → 1298 (COMBIN_COMBINA C5, C16), +844 new bit-exact cells.
Semantics: numbers.md "COMBIN is inexact".
