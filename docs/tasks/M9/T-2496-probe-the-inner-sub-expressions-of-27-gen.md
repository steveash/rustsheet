---
id: T-2496
title: "Probe the inner sub-expressions of 28 gen-m9 financial cells of the T-2257 sweep whose divergence is not traced"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Each of these 28 gen-m9 financial cells has its innermost diverging sub-expression identified from Excel values (a probe case that evaluates the sub-expressions with the same Data sheet), and is then owned by a fix task (new or existing), with the quirks recorded in docs/excel-semantics/.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. The triage traced 937 of the 965 visibly-off cells to a family by evaluating every
sub-expression in the engine; these 28 are left: their Excel value is an error (`#N/A`, `#REF!`, `#CALC!`, `#SPILL!`,
`#DIV/0!`) that the outer financial function cannot produce and that none of its arguments has in ours, or a number or error
that needs an inner function to differ. Most involve an inner function outside the financial lane (XMATCH, PERCENTRANK.INC/
.EXC, ZTEST/Z.TEST, PROB, DSTDEVP/DCOUNT/DCOUNTA/DSUM, GROWTH, QUARTILE over EXPAND, CHISQ.INV, WEIBULL.DIST, F.DIST, PIVOTBY,
TOCOL, RSQ under EFFECT) and a few a financial inner call (PRICEMAT over FILTER is `#CALC!`, `PV(-1+2^-52,91,-421)` and `NPV(-1,..)` inside
MDURATION and YIELD's COUPDAYBS). Method: copy each case's Data sheet into a probe case (excel-probe skill), one formula per
sub-expression of each cell, fill it through excel-graph, compare with the engine, then file or extend the owning task for
each family found (decision 0083: `campaign: visible`). T-2436 and T-2475 do the same for the statistical sweeps; reuse their
probe layout.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 28 known-failure lines in eval/baseline/gen-m9.txt, 28 visibly off.
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A334 `=COUPDAYS(39309,44860,NPV(1E+300,Data!D5:D8,Data!D4:D5),2)`: Excel #DIV/0!, ours #NUM!
- gen-m9/31474c20fc1dafd6 Calc!A198 `=NPV(SUM(Data!A7:A9,DSUM(Db!A1:D9,1,Db!F1),Data!B6:B12),Data!E9:E9)`: Excel #VALUE!, ours -2.4999999999999996e-15
- gen-m9/762cd0e4ec9a7aa9 Calc!A153 `=PDURATION(0.1,DCOUNT(Db!A2:D9,3,Data!A8:D8),34623)`: Excel #VALUE!, ours 92.77954973759424
- gen-m9/bc04c1f478c9896c Calc!A237 `=NOMINAL(TOCOL(Data!Z1,3,ISERROR(#VALUE!)),3)`: Excel #CALC!, ours #NUM!
- gen-m9/c0681a8b150ce301 Calc!A101 `=MDURATION(36880,46327,PV((-1+2^-52),91,-421),7,"2",3)`: Excel #DIV/0!, ours #NUM!
- gen-m9/d0854366830dcdee Calc!A298 `=DOLLARFR(WEIBULL.DIST(7,1E+8,Data!D5,"TRUE"),32)`: Excel #NUM!, ours 1.0
Cells:
- gen-m9/01523d247594952e.case.json (2): Calc!A51, A374
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A334
- gen-m9/23d82b62e9a85bdd.case.json (1): Calc!A9
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A198
- gen-m9/3aa168259335be54.case.json (1): Calc!A94
- gen-m9/5422bdbf173fc233.case.json (2): Calc!A227, A294
- gen-m9/762cd0e4ec9a7aa9.case.json (4): Calc!A78, A153, A169, A343
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A395
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A396
- gen-m9/9d9ceba257f8001b.case.json (2): Calc!A192, A288
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A149
- gen-m9/b6a02893a8d6b151.case.json (2): Calc!A78, A245
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A178
- gen-m9/bc04c1f478c9896c.case.json (2): Calc!A237, A275
- gen-m9/c0681a8b150ce301.case.json (3): Calc!A101, A136, A290
- gen-m9/d0854366830dcdee.case.json (2): Calc!A183, A298
- gen-m9/d912671baa1364aa.case.json (1): Calc!A347

Moved from T-2482 (2026-10-24): YIELD's maturity is `ODDFPRICE(SUM(..)<123456793.56>,46471,..,IFNA("","1.5")<"">,..)`,
ours `#VALUE!` (the text rate). Excel's YIELD is `#NUM!`, so its ODDFPRICE is likely `#NUM!` from the settlement past
9999-12-31, read before the rate as PRICE does (financial.md "PRICE, YIELD, DURATION and MDURATION check order"). Probe
the inner ODDFPRICE. Fixed by T-2483 (2026-10-24): a settlement past 9999-12-31 is `#NUM!` as ODDFPRICE reads it
(financial.md "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order"); the line below is gone from the baseline.
Owned gen-m9 cells (T-2482, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 visibly off.
- gen-m9/3c06f9298f65a3ba Calc!A309 `=YIELD(38778,ODDFPRICE(SUM(Data!D11,Data!D4:D8),46471,BYCOL(Data!D4,LAMBDA(acc,1000000000000000.0)),38622,IFNA("","1.5"),IPMT(2,1,150,481849),109,2),IFNA((1e-15>Data!A3),(Data!E11-Data!B6)),101,103,2)`: Excel #NUM!, ours #VALUE!

Moved from T-2487 (2026-10-25): four T-bill cells whose error differs from Excel's in an inner call, not in the T-bill
argument order; probes/tbill-error-order-inner-calls (each case's Data sheet copied, excel-graph) already traces them:
`T.TEST(..)` is `#VALUE!` in Excel (ours `#NUM!`), `PPMT(2*B11,8,360,-100,926)` 0 (ours 2E17), `QUARTILE(<text cell>,..)`
`#NUM!` (ours `#VALUE!`), `BIN2DEC(<text cell>)` `#NUM!` (ours `#VALUE!`). File or extend the fix task for each
(financial.md "T-bill settlement below serial 1").
Owned gen-m9 cells (T-2487, excel-graph, 2026-10-25): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 visibly off.
- gen-m9/3c06f9298f65a3ba Calc!A312 `=TBILLEQ(39717,(T.TEST(Data!E11,Data!A6:C12,2,Data!$D8)*Data!A$2%),(Data!E$7-AVERAGE(Data!C8)))`: Excel #VALUE!, ours #NUM!
- gen-m9/81bcdc0bec0c8e6b Calc!A251 `=TBILLEQ(PPMT((2*Data!B11),8,360,(-100),926),COVAR(Data!C3:E12,Data!B3:B9),Data!C3)`: Excel #N/A, ours #NUM!
- gen-m9/b72d952c5f52cb4d Calc!A241 `=TBILLYIELD(COUNT(-2),STDEV.S(QUARTILE(Data!C7,Data!$E9),Data!D8:D8),94)`: Excel #NUM!, ours #VALUE!
- gen-m9/d0854366830dcdee Calc!A137 `=TBILLPRICE(COUNT(Data!B7:B10)%,59,SUM(Data!E6:E8,BIN2DEC(Data!A6)))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A312
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A251
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A241
- gen-m9/d0854366830dcdee.case.json (1): Calc!A137

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2482): the known-failure line listed under "Moved from T-2482" disappears from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2257): these 28 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline. 16 have left the baseline; the other 12
      are transferred to their owning fix tasks' visible-parity criteria (Outcome), as T-2436 did.
- [x] Visible parity (gen-m9, T-2487): the 4 known-failure lines under "Moved from T-2487" disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline. Transferred to
      their owning fix tasks' visible-parity criteria (Outcome).
- [x] Every cell above is listed in a fix task's Context (new or extended), or explained in financial.md as a quirk with its owning task; the probe case's cells are in eval/ and gated or baselined.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the functions (the tasks this one files).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
16 of the 32 owned cells (28 from T-2257, 4 from T-2487) already pass on main: later merges fixed them, and their lines
are gone from eval/baseline/gen-m9.txt. For the other 16, probe `probes/t2496-fin-subexpressions` (99 cells, excel-graph,
2026-10-25) copies each case's Data sheet (D01-D13) and puts every call and compound argument of each owned cell in its own
row, read through a TYPE/shape summary. Column B names the cell and the sub-expression. The four T-bill cells were already
traced by probes/tbill-error-order-inner-calls. Each innermost divergence now sits in its owner's Context, under a
"T-2496 probe" block that has its own visible-parity criterion:
- existing tasks: T-2470 Z.TEST (2), T-2468 PROB (2), T-2427 FREQUENCY with no numeric bin gives 2 rows, so the enclosing
  call lifts and spills (2), T-2493 NPV at -1 and 1E+300 (2), T-2469 T.TEST blank sample (1), T-2490 PPMT at a huge rate (1),
  T-2344 QUARTILE of a TRUE cell (1), T-2435 WEIBULL.DIST overflow (1), T-2544 BIN2DEC of text (1);
- new tasks: T-2656 XMATCH search_mode within 15 digits of 1 (1, M6), T-2657 PV near a rate of -1 and IRR from a guess of
  42 (2, opus).
Quirks: financial.md "Financial cells of the T-2496 sub-expression probe", lookup.md "XMATCH: a search_mode within 15
digits of 1 reads 1", and statistics.md "T.TEST of a blank one-cell sample and WEIBULL.DIST overflow".
Eval delta: the probe's 32 cells that ours fails today are new-coverage lines in eval/baseline/probes.txt (the divergences
those tasks own). 0 regressions, gen-m9 unchanged. The visible-parity lock is unchanged at 6,039; 32 of the 5,684
visibly-off cells are in the new case.
