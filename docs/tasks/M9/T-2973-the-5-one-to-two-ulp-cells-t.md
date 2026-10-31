---
id: T-2973
title: "The 5 one-to-two-ulp cells T-2733's bond sum order moved: DURATION/MDURATION B7, B43, C7, C43, gen-m9 5ded7594e597e31e A123"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2733]
defer: phase-1b
---

## Goal
The five cells that were bit-exact before T-2733's sum order are bit-exact again, without losing T-2733's fixes.

## Context
T-2733 (decision 0083's ulp-regression allowance) moved these cells by 1 or 2 ulp; none is visibly off (Excel vs ours):
- probes/bond-extreme-coupon-yield Calc!B7 `DURATION(38136,44737,100,1,1,0)`: 1.0721863067796484 vs ...482 (1 ulp);
  B43 `DURATION(38136,44737,100,1,2,0)`: 1.0722166274036073 vs ...076 (1 ulp); C7 `MDURATION(38136,44737,100,1,1,0)`:
  0.5360931533898242 vs ...241 (1 ulp); C43 `MDURATION(38136,44737,100,1,2,0)`: 0.7148110849357382 vs ...384 (2 ulp).
- gen-m9/5ded7594e597e31e Calc!A123 `DURATION(37952,44607,123456789.123,Data!$C3,1,2)`: 2.1997586986000424 vs ...043 (1 ulp).
probes/yield-newton-domain P!B7 (YIELD) also moved by an ulp before T-2733 merged `main`; it passes on top of
T-3215's YIELD start, so this task does not own it.
DURATION's redemption exponent written `N-1+DSC/E` instead of `DSC/E+N-1` moves 3 more cells (coupdays-feb-month-end
S!G20, H20, date-system-1900-coupon-bound S!M1), so it stays as is.

## Acceptance criteria
- [ ] The five cells pass with no new known-failure lines in any baseline, or the candidates scored are recorded in
      financial.md (time-boxed, decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2972's cells.

`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits.
