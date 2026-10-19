---
id: T-1757
title: "INTRATE, PRICEDISC and YIELDDISC accept settlement serial 0 (RECEIVED, PRICEMAT, YIELDMAT do not)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
INTRATE, PRICEDISC and YIELDDISC return a number when settlement is serial 0, as Excel does.

## Context
Excel: INTRATE(0,43466,1000,1100)=0.0008403165192222404, PRICEDISC(0,43466,0.05,100)=-495.0138888888889, YIELDDISC(0,43466,95,100)=0.00044227185222223174; we return #NUM!. RECEIVED, PRICEMAT and YIELDMAT give #NUM! for settlement 0 in Excel too. Failing cells: probes/discount-security-modes.case.json Calc!A21, A43, A88. See docs/excel-semantics/financial.md (discount-security settlement 0).

## Acceptance criteria
- [ ] Calc!A21, A43 and A88 of probes/discount-security-modes pass (bit-exact or within the existing last-digit class); RECEIVED/PRICEMAT/YIELDMAT #NUM! cells (A65, A110, A133) keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit misses (T-1374).

## Outcome
Already fixed by T-1409 (YEARFRAC accepts serial 0 on basis 0, which these functions call). Calc!A21, A43 and A88 now return numbers and differ from Excel only in the last digit (NumericClose/NumericUlp, T-1374 class), so they stay in the baseline. Added unit test `discount_security_settlement_zero` (INTRATE/PRICEDISC/YIELDDISC compute, RECEIVED #NUM!) and corrected the `financial.md` "Ours" line. Eval delta: none.
