---
id: T-1155
title: "XIRR with a zero first value is 0.1/2^25, and date-0 first flows converge"
milestone: M1
lane: functions-financial
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
XIRR with a zero first value gives Excel's 2.9802322387695314E-09, and XIRR with a first flow on date 0 gives Excel's root instead of #NUM!.

## Context
Found in T-1124 (financial.md, "XIRR reads empty cells as 0 and accepts earlier dates"). A zero first
value gives 0.1 / 2^25 whatever the date (probes/xirr-mismatched-empties X3, X4; xirr-early-dates A10, A11),
which suggests Excel's solver halves the guess 25 times when f is flat or the first flow is 0; probe other
guesses (XIRR(...,0.2), 1) to find the rule. -1000 on date 0 and 1100 on 45365 is 0.000767144560813904 in
Excel (X5, A8), but `compute_xirr` (functions/financial/financial_util.rs) returns #NUM!: Newton from 0.1
diverges over a 124-year span and the bisection hits a non-finite XNPV at -0.9999.

## Acceptance criteria
- [ ] The zero-first-value rule is probed (other guesses) and recorded in financial.md.
- [ ] xirr-mismatched-empties X3, X4, X5 and xirr-early-dates A8, A10, A11 match Excel to the last-digit tolerance of the early stop (or exactly), with no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
Matching these cells bit for bit needs Excel's whole XIRR iteration: the zero-first-value result and
the date-0 root are points of Excel's bracket-and-bisect search, not of a root-finder the engine can
patch locally. The search structure is recorded in financial.md, "XIRR is a doubling search and a
bisection, not Newton": every result lies on a g * j / 2^k lattice. A simulation reproduces 28 of 34
positive-root cells but not guess 1, negative roots or these task's cells. Fitting the stopping rule and
the negative-root search is T-0916 (opus, M9). Unblock when T-0916 lands a fitted solver: X3-X5 and
A8, A10, A11 should then follow from the same model, along with the zero-first-value probes for other
guesses, which are still to do. Simulation scripts are not kept. The model is described in full in
financial.md.
