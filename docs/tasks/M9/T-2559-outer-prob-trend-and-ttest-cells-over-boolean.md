---
id: T-2559
title: "Outer PROB, TREND and TTEST cells over boolean F/beta results still off (T-1849 residue)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The gen-m9 cells T-1849 left visibly off match Excel's type and error, and the ulp-level ones are re-checked.

## Context
T-1849 made the F, beta, hypergeometric, Weibull and STANDARDIZE kernels read a boolean as a number. These cells still differ:
- gen-m9/1b31092a0d79e5f1 Calc!A342 (PROB), gen-m9/583ca2baea0bb30a Calc!A211 (TREND), gen-m9/928d08e3e1eecb5f Calc!A157 (TTEST over F.DIST): the outer function gives the wrong error.
- ulp-level: 0a9226016e8f5f33 A325, a5828d3925575713 A281, a790a4d4a6a82756 A121, c6cfda6bd10cae93 A183, ee2497a5e683c5d9 A356.

## Acceptance criteria
- [ ] These cells leave eval/baseline/gen-m9.txt (`make eval-baseline`), or each remaining one has a reason in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
