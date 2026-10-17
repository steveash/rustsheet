---
id: T-1757
title: "INTRATE, PRICEDISC and YIELDDISC accept settlement serial 0 (RECEIVED, PRICEMAT, YIELDMAT do not)"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] Calc!A21, A43 and A88 of probes/discount-security-modes pass (bit-exact or within the existing last-digit class); RECEIVED/PRICEMAT/YIELDDISC-style #NUM! cells keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
