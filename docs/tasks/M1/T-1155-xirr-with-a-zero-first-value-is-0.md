---
id: T-1155
title: "XIRR with a zero first value is 0.1/2^25, and date-0 first flows converge"
milestone: M1
lane: functions-financial
status: done
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
- [x] (moved to T-1591) The zero-first-value rule is probed (other guesses) and recorded in financial.md.
- [x] xirr-mismatched-empties X3, X4, X5 and xirr-early-dates A8, A10, A11 match Excel to the last-digit tolerance of the early stop (or exactly), with no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done by other tasks; closed by T-0059. The cells pass on main since T-1583 (#813) and T-0916 (#914):
eval/baseline/probes.txt has only coverage lines for probes/xirr-early-dates (16) and
probes/xirr-mismatched-empties (14), no known failures. The rule is in docs/excel-semantics/financial.md,
"XIRR is a bracketing bisection, not Newton": with a zero first value the end replaced follows the first
value's sign, so the result is 0.1/2^25. Probing other guesses with a zero first value (0.2 and 1) moved
to T-1591. This task's own draft PR #409 (a partial bracket-and-bisect model, 28 of 34 cells) is
superseded by that entry and was closed.
