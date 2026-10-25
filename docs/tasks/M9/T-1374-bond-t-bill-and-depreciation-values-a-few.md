---
id: T-1374
title: "Bond, T-bill and depreciation values a few ulps off Excel (probes/fin-argument-modes base rows)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The bond, T-bill, depreciation and cumulative-payment functions return Excel's bits for the probes/fin-argument-modes base rows.

## Context
YIELD's solver (T-1525, 2026-10-14): see `docs/excel-semantics/financial.md`, "YIELD stops short of PRICE's root";
probes/solver-iteration-traces Y!C:E (24 cells) and the basis-0 multi-coupon cells of yield-solver-sweep are scored
by `uv run rustsheet-fitlab yield` (90 cells, best candidate 2/90 bit-exact); the fit continues in T-1533. YIELD with one
coupon period left is closed form (T-1532).
probes/fin-argument-modes (excel-graph, 2026-10-11, T-1084). Base rows that miss:
- 1 to a few ulps off: CUMIPMT H72, DB H88, DDB H95, DURATION H117, INTRATE H139, MDURATION H163, ODDFPRICE H189,
  PRICE H251, YIELDDISC H352.
- Tens of ulps off: CUMPRINC H80 (about 37) and EFFECT H125 (about 32).
- Off at about 1e-10 to 1e-12 relative, so probably an iteration stopping point rather than an evaluation order: ODDFYIELD H200,
  YIELD H343. (TBILLEQ H313 and the other TBILL cells moved to T-1475 on 2026-10-13.)
- Added by T-1842: probes/fin-yield-odd-lift A1 and I5 (YIELD, about 1e-13 relative), A3 and I7 (ODDFPRICE, a few
  ulps), A5 and I9 (ODDFYIELD, about 1e-10 relative): the same base values as above, reached through the lift.
- Moved from T-1493 on 2026-10-13: YIELD A8-E8, A19-E19, A30-E30 and DURATION A20/C20 of
  probes/date-system-1900-financial-readers and its -shifted twin, and the 4 DURATION ulp cells of
  probes/coupdays-feb-month-end. probes/yield-solver-sweep (66 cells, excel-graph) is the YIELD sweep: Excel stops
  below the root, the stopping point does not move with the dates for the same day counts but does differ between
  two bonds with equal day counts and different YEARFRAC (docs/excel-semantics/financial.md, "YIELD stops short of
  PRICE's root").
- Added by T-1845: probes/fin-remaining-later-argument-ranges K11 `INDEX(CUMIPMT({0.01;0.02},360,100000,1,12,0),2)`
  (-23997.734423612783, ours an ulp off): the 12-period CUMIPMT sum, kept up to 32 periods by T-1845; the closed form
  past 32 periods is T-1890's.
The same DURATION, MDURATION and PRICE base values recur as the intersected controls of
probes/fin-remaining-later-argument-ranges I85-I103 and I165-I171 (T-1822), and so do the CUMIPMT, CUMPRINC, ODDFPRICE,
ODDFYIELD and YIELD ones there (I65-I84, I105-I136, I173-I184).
The annuity family (FV, PMT, PPMT, NPV) and the solvers (IRR, RATE, XIRR) are T-0901's and T-0916's, not this task's.
ODDFYIELD's solver (H200, fin-yield-odd-lift A5/I9, the ODDFYIELD controls of fin-remaining-later-argument-ranges and the
PRICE_YIELD.xlsx K column) moved to T-2007 (gap triage 2026-10-19); this task keeps ODDFPRICE.
Gap triage 2026-10-19 split three families out: the discount securities (INTRATE H139, YIELDDISC H352 and the
probes/discount-security-modes acceptance line below) to T-2032; DB H88 and DDB H95 to T-2033 (fitted there);
EFFECT H125 to T-2034. This task keeps CUMIPMT, CUMPRINC, DURATION, MDURATION, ODDFPRICE and PRICE.
Gap triage 2026-10-20: PRICE and ODDFPRICE's coupon discounting (libm `powf` where Excel uses its `^`;
the `^` model gives Excel's bits on PRICE_YIELD.xlsx H2-H5, H8-H11) moved to T-2207 with their visibly-off
cells; this task keeps whatever PRICE/ODDFPRICE residue T-2207 leaves.
One probe point per function is not enough to fit a formulation: first add an oracle sweep per function family
(excel-probe skill; probes-numeric like T-0942), then fit. Split per family if larger than M.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): PRICE, DURATION, MDURATION and PRICEMAT values of the financial sweep visibly off (`DURATION(36678,44307,46.05170259237739,0.2,4)` Excel 4.785592595796413, ours 4.785592595773397; `PRICE(43830,45293,100,278,50,4,1)` -133.76680342333566, ours ...657; `PRICEMAT(38879,45322,33651,2,2.14379,..)` -2696.0973159209293, ours ...5355), and last-digit cells of the bond, CUMIPMT, CUMPRINC, YIELD, SLN, DOLLARDE, DOLLARFR, FVSCHEDULE, INTRATE and PRICEMAT calls.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 6 known-failure lines in eval/baseline/gen-m9.txt, 6 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A279 `=PRICE(38136,44737,1E+300,7,100,2)`: Excel 8.95449454935698e+300, ours 8.95449454935699e+300
- gen-m9/30e20ba3302fea8b Calc!A90 `=MDURATION(39697,44449,CHOOSE(3,Data!D3,-Data!$E10,EDATE(58318,22)),DATEVALUE("15-Mar-2021"),2,Data!$A8)`: Excel 5.029680055838246e-07, ours 5.029680055838228e-07
- gen-m9/30e20ba3302fea8b Calc!A310 `=PRICEMAT(39955,45657,34754,FINV(1E-100,2,3),0.05,1)`: Excel 1.7583339520153275e+69, ours 1.7583339520153015e+69
- gen-m9/3c06f9298f65a3ba Calc!A326 `=DURATION(36678,44307,CHISQ.INV(1-1E-10,2),Data!E5,4)`: Excel 4.785592595796413, ours 4.785592595773397
- gen-m9/a91bad4e84f49739 Calc!A142 `=PRICE(43830,45293,100,CEILING.MATH(PERCENTILE.INC(Data!C2:D11,Data!C11),DATEDIF(100,8589,"M")),50,4,1)`: Excel -133.76680342333566, ours -133.76680342333657
- gen-m9/b6a02893a8d6b151 Calc!A260 `=PRICEMAT(38879,45322,33651,2,YIELD(39824,45657,SUM(Data!B$3,0.1),96,106,2,4))`: Excel -2696.0973159209293, ours -2696.0973159215355
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A279
- gen-m9/30e20ba3302fea8b.case.json (2): Calc!A90, A310
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A326
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A142
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A260

Last-digit cells of the same sweep (not visibly off; re-check them after the fix):
Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 53 known-failure lines in eval/baseline/gen-m9.txt, 53 last-digit (not visibly off).
Examples:
- gen-m9/0a957119475760c5 Calc!A265 `=CUMIPMT(0.05/12,334,1,12,22,1)`: Excel -0.044598593849635604, ours -0.0445985938496356
- gen-m9/67a4c03afd63f988 Calc!A91 `=DOLLARFR(BESSELJ(700,2),28)`: Excel 0.0017843081440053886, ours 0.001784308144005389
- gen-m9/7d2979fba2c95d64 Calc!A318 `=DOLLARFR(T.DIST.RT(0.5,9),25)`: Excel 0.07863391247825333, ours 0.07863391247825331
Cells:
- gen-m9/01523d247594952e.case.json (1): Calc!A225
- gen-m9/0a957119475760c5.case.json (2): Calc!A19, A265
- gen-m9/1b60bf99d71424e7.case.json (3): Calc!A112, A181, A381
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A359
- gen-m9/31474c20fc1dafd6.case.json (4): Calc!A61, A201, A311, A319
- gen-m9/3aa168259335be54.case.json (4): Calc!A70, A119, A153, A348
- gen-m9/3c06f9298f65a3ba.case.json (2): Calc!A175, A211
- gen-m9/5c6a23c4f6597f1c.case.json (2): Calc!A7, A150
- gen-m9/6376e27be7a05871.case.json (1): Calc!A120
- gen-m9/67a4c03afd63f988.case.json (4): Calc!A91, A208, A332, A355
- gen-m9/762cd0e4ec9a7aa9.case.json (1): Calc!A64
- gen-m9/79d87fc527927626.case.json (3): Calc!A156, A163, A254
- gen-m9/7d2979fba2c95d64.case.json (2): Calc!A190, A318
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A59
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A152
- gen-m9/9a1fa5366bc50918.case.json (3): Calc!A165, A228, A312
- gen-m9/9d9ceba257f8001b.case.json (2): Calc!A262, A395
- gen-m9/a91bad4e84f49739.case.json (2): Calc!A28, A389
- gen-m9/abb7d565af89b849.case.json (1): Calc!A73
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A274
- gen-m9/b72d952c5f52cb4d.case.json (2): Calc!A159, A179
- gen-m9/b73c6cc96e093f03.case.json (4): Calc!A12, A41, A111, A306
- gen-m9/ba4ddb817a8e700e.case.json (2): Calc!A116, A227
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A292
- gen-m9/d912671baa1364aa.case.json (1): Calc!A104
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A23, A317

Moved from T-2487 (2026-10-25): a last-digit T-bill cell, visibly right; whether TBILLYIELD's order or its inner F.INV
gives the ulp is unprobed.
Owned gen-m9 cells (T-2487, excel-graph, 2026-10-25): 1 known-failure line in eval/baseline/gen-m9.txt, 1 last-digit (not visibly off).
- gen-m9/e8d55ec393460bb7 Calc!A91 `=TBILLYIELD(COS(KURT(Data!B1:B9,Data!A9:A11,Data!D11:D11)),(Data!E3*SORT("1.5")),F.INV(ABS(Data!E5),4,3))`: Excel 150510.9757994007, ours 150510.97579940068
Cells:
- gen-m9/e8d55ec393460bb7.case.json (1): Calc!A91

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 6 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 53 T-2257 last-digit lines in Context are re-checked after the fix; the Outcome says how many pass.
- [ ] A probes-numeric sweep for the bond and depreciation families is oracle-filled.
- [ ] The base rows above pass, or the Outcome records which formulation was ruled out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FV, PMT, PPMT, NPV (T-0901); IRR, RATE, XIRR, MIRR (T-0916).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
