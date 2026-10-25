---
id: T-2496
title: "Probe the inner sub-expressions of 28 gen-m9 financial cells of the T-2257 sweep whose divergence is not traced"
milestone: M9
lane: research
status: todo
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

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2482): the known-failure line listed under "Moved from T-2482" disappears from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): these 28 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Every cell above is listed in a fix task's Context (new or extended), or explained in financial.md as a quirk with its owning task; the probe case's cells are in eval/ and gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the functions (the tasks this one files).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
