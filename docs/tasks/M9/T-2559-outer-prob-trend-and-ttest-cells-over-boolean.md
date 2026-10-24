---
id: T-2559
title: "Outer PROB, TREND and TTEST cells over boolean F/beta results still off (T-1849 residue)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The 10 gen-m9 cells T-1849 left off match Excel's type, error and text, or are within half a unit of the 15th
significant digit.

## Context
T-1849 made the F, beta, hypergeometric, Weibull and STANDARDIZE kernels read a boolean as a number. Still failing:
- Wrong error: gen-m9/1b31092a0d79e5f1 Calc!A342 (PROB, Excel `#N/A`, ours `#VALUE!`); 583ca2baea0bb30a A211 (TREND,
  `#VALUE!` vs ours `#REF!`); 928d08e3e1eecb5f A157 (TTEST, `#VALUE!` vs `#NUM!`); 7bd219ec0c0c5e22 A93 (FTEST over
  BETADIST, `#NUM!` vs `#VALUE!`, the FTEST error-argument gap of T-2093).
- Visibly off numerically: a790a4d4a6a82756 A121 `F.DIST.RT`; c6cfda6bd10cae93 A183 `F.INV.RT`; a612e89321a6df9c A205
  `BETAINV(1E-20,7,NOMINAL(0.05,4),Data!B11)`.
- Within 4 ulp (residue, no campaign): 0a9226016e8f5f33 A325, a5828d3925575713 A281, ee2497a5e683c5d9 A356.

## Acceptance criteria
- [ ] The 4 wrong-error and 3 visible-numeric cells leave eval/baseline/gen-m9.txt (`make eval-baseline`).
- [ ] A typed TRUE in the unprobed arguments of T-1849 is probed (excel-probe) and the statistics.md entry updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
