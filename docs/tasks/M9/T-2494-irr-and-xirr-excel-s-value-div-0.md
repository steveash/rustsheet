---
id: T-2494
title: "IRR and XIRR: Excel's #VALUE!, #DIV/0! and #NUM! for a single value, a huge guess, zero flows and no root (gen-m9, 21 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
IRR returns Excel's `#VALUE!`, `#DIV/0!` or `#NUM!` where ours returns a root or another error, and XIRR returns Excel's value at a huge guess.

## Context
Gap triage of the T-2257 gen-m9 financial sweep.
- A single value is `#VALUE!`: `IRR(CELL("row",Data!D$2),0.5)`, `IRR(COUNTBLANK(..),1E+300)` (ours `#NUM!`).
- A guess of 1E+300 (or 2.5E29) is `#VALUE!`: `IRR(Data!E4:E8,1E+300)`, `IRR(Data!C5:D6,1E+300)` (ours 30.13),
  `IRR({0,0,-100,50,60},1E+300)` (ours 0.0639), `IRR(Data!B8:C8,VARPA(..))`.
- Zero flows: `IRR(Data!A1:D12,0.05)` is `#DIV/0!` (ours `#NUM!`).
- No root where ours finds one near -1 or far out: `IRR(Data!B10:E12,1)` (ours -0.99995), `IRR(Data!C9:D12)` (ours -0.860),
  `IRR(Data!A1:C3)` (ours 31622776.09), `IRR(Data!B3:E9,-0.9999999999)` (ours -0.9999999999), 11 cells: Excel's search gives up
  where ours' fallback goes on (financial.md "The IRR and XIRR solver caps", "IRR is a secant in v = 1/(1 + r)").
- An error argument whose Excel value is another error: `IRR(CHITEST(..)-..,..)` `#VALUE!` (ours `#N/A`),
  `IRR(HSTACK(..)*..,1)` `#VALUE!`, `IRR(1901+UNIQUE(..),INDEX(..)<#REF!>)` `#REF!`, `IRR(UNIQUE(..)<#REF!>,..)` `#VALUE!`.
- XIRR: `XIRR({-100,10,20,30,40},{0,1,59,60,61},123456789.123)` is 3.4266142457661796E-09 (ours `#NUM!`).
The data cells are the Data sheets of the listed cases (`make eval-show`). docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)". T-1686 (probe of the IRR/XIRR caps
and fallback) may answer the no-root rule; read its Outcome first if it is done.
Code: `fn_irr`, `fn_xirr` in crates/rustsheet-engine/src/functions/financial/mod.rs, `compute_irr` in financial_util.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 21 known-failure lines in eval/baseline/gen-m9.txt, 21 visibly off.
Examples:
- gen-m9/1b60bf99d71424e7 Calc!A215 `=IRR(Data!E4:E8,1E+300)`: Excel #VALUE!, ours #NUM!
- gen-m9/5c6a23c4f6597f1c Calc!A153 `=IRR((YEAR(Data!D$11)+UNIQUE(Data!B2:E10,TRUE,TRUE)),INDEX(Data!A3:B9,2))`: Excel #REF!, ours #VALUE!
- gen-m9/79d87fc527927626 Calc!A26 `=IRR(Data!C5:D6,1E+300)`: Excel #VALUE!, ours 30.126729201490246
- gen-m9/9a1fa5366bc50918 Calc!A400 `=IRR(Data!A1:C3)`: Excel #NUM!, ours 31622776.086705893
- gen-m9/a91bad4e84f49739 Calc!A350 `=XIRR({-100,10,20,30,40},{0,1,59,60,61},MAX(Data!E4:E8))`: Excel 3.4266142457661796e-09, ours #NUM!
- gen-m9/bc04c1f478c9896c Calc!A56 `=IRR(Data!A1:D12,0.05)`: Excel #DIV/0!, ours #NUM!
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A293
- gen-m9/1b60bf99d71424e7.case.json (2): Calc!A117, A215
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A277
- gen-m9/3aa168259335be54.case.json (1): Calc!A1
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A251
- gen-m9/5422bdbf173fc233.case.json (1): Calc!A63
- gen-m9/5c6a23c4f6597f1c.case.json (3): Calc!A104, A153, A349
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A282
- gen-m9/6376e27be7a05871.case.json (1): Calc!A9
- gen-m9/79d87fc527927626.case.json (1): Calc!A26
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A400
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A350
- gen-m9/abb7d565af89b849.case.json (1): Calc!A254
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A236
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A56
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A22
- gen-m9/d912671baa1364aa.case.json (1): Calc!A158
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A395

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 21 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for a scalar value, a 1E+300 guess and all-zero flows.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's returned digits (T-2131) and IRR over an array with an error element (T-2390).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
