---
id: T-2732
title: "Oracle sweep: PRICE, DURATION and MDURATION at extreme coupon rates and yields (coupon 1E+2..1E+300, yield 1..1E+5) for T-2733"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An excel-graph probe gives T-2733 enough Excel points to fit PRICE's and MDURATION's evaluation where both the
coupon rate and the yield are far above 1. Today only three gen-m9 cells cover that region, too few to fit an
operation order (the gap-triager rule: no blind fitting).

## Context
Gap triage 2026-10-25. T-2733 owns three visibly-off gen-m9 cells, where both arguments are extreme:
- gen-m9/0a957119475760c5 Calc!A279 `PRICE(38136,44737,1E+300,7,100,2)`: Excel 8.95449454935698e+300,
  ours 8.95449454935699e+300.
- gen-m9/a91bad4e84f49739 Calc!A142 `PRICE(43830,45293,100,278,50,4,1)` (the coupon comes from CEILING.MATH(..) and is
  278 per T-1374): Excel -133.76680342333566, ours -133.76680342333657.
- gen-m9/30e20ba3302fea8b Calc!A90 `MDURATION(39697,44449,EDATE(58318,22),DATEVALUE("15-Mar-2021"),2,Data!$A8)` (coupon
  about 58988, yield 44270): Excel 5.029680055838246e-07, ours 5.029680055838228e-07.
Existing evidence covers each extreme alone, and all of it passes. probes/bond-huge-yield-overflow (T-2488, 78
cells) has yields up to the overflow at a 5% coupon, and coupons up to 9.99999999999999E+307 at a 5% yield.
probes/fin-mduration-rri-fit (T-1975) has ordinary rates only. Rule today: financial.md "PRICE and ODDFPRICE discount
with Excel's `^`".

Design. Use the excel-probe skill; probes go in eval/suites/probes/cases, not probes-numeric, so the lines are gated.
- PRICE, DURATION and MDURATION on one grid: coupon rate in {10, 100, 1E+4, 1E+10, 1E+100, 1E+300}; yield in
  {1, 7, 42, 278, 1E+3, 44270}; frequency 1, 2 and 4; basis 0 and 1.
- Settlement both on a coupon date and mid-period. Include the three cells' own dates and the 17-coupon (A142) and
  13-year (A279) terms.
- Companion cells that isolate the pieces: `(1+yld/frequency)^t` for the exponents the bond uses, the coupon-sum term
  alone (redemption 0, where valid), and the redemption term alone (rate 0).
- Keep the probe within the oracle's cell budget, about 600 cells. Fill through excel-graph and check the canary.

## Acceptance criteria
- [ ] The probe case and its excel-graph sidecar are committed, and every cell the engine fails is a new line in
      eval/baseline/probes.txt (written by `make eval-baseline`, never by hand) that T-2733 owns. That task's Context lists them with Excel vs ours, by function.
- [ ] financial.md gains an observation paragraph under "PRICE and ODDFPRICE discount with Excel's `^`": where the
      engine leaves Excel's 15 digits, a high-precision (mpmath, 50 digits) value of each cell, and whether Excel or
      the engine is closer to it. That decides T-2733's stage: accuracy or visible.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-2733). YIELD and ODDFYIELD solvers (T-1592, T-2007).
