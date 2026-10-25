---
id: T-2482
title: "PRICE, YIELD, DURATION and MDURATION: Excel's argument check order under error and text arguments (gen-m9, 50 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
PRICE, YIELD, DURATION and MDURATION give Excel's result when one argument is an error, text or a boolean and another is out of its domain: Excel reads the arguments in order and an earlier invalid date, rate or yield is `#NUM!` before a later argument's error or text is looked at, and an earlier error or text before a later domain check.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. In these cells ours returns a later argument's error (or the text `#VALUE!`) where
Excel returns `#NUM!` for an earlier argument, or the reverse: `PRICE(36917,-0.5,HOUR(..)<#VALUE!>,..)` (maturity before
settlement, then an error coupon) is `#NUM!`; `DURATION(2958466,46410,N(..),<#VALUE!>,4,0)` is `#NUM!` (a settlement past
9999-12-31 decides first); `PRICE(-0.0001,VALUE("Hello World"),-1+2^-52,1E+300,1E+6,4,7)` (a negative settlement first) is `#NUM!`, not `#VALUE!`; in
`YIELD(38507,PRODUCT(RSQ(..))<#N/A>,1E-300,..)`, `#VALUE!` in Excel, the paired RSQ may be what differs (T-2426, T-2431). The same reader rule is
VERIFIED for the COUP* functions ("Analysis ToolPak securities functions check frequency and dates first", T-1406); for these
four it is inferred from the generated cells (docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)"), so confirm each argument position with a small probe
(excel-probe skill) before changing the reader, and keep the order inside the shared coupon reader so COUP* stay green.
A few cells nest a call whose own value may differ in Excel (ODDFPRICE, ACCRINT, BESSELJ, NETWORKDAYS under an argument); if a
probe shows the inner call is the cause, move the cell to T-2496 and say so in the Outcome.
Code: `fn_price`, `fn_yield`, `fn_duration`, `fn_mduration` and their argument reader in
crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (`coupon_args`).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 50 known-failure lines in eval/baseline/gen-m9.txt, 50 visibly off.
Examples:
- gen-m9/1dce4774f5c0e736 Calc!A60 `=YIELD(SUM(0.5,Data!A8:A11),(1/LEN("İ")),Data!B8,92,Data!$B9,VAR(Data!E5:E8),1)`: Excel #NUM!, ours #VALUE!
- gen-m9/3c06f9298f65a3ba Calc!A113 `=YIELD(38507,PRODUCT(RSQ(Data!E11:E11,Data!A9:D9)),1E-300,94,104,2,0)`: Excel #VALUE!, ours #N/A
- gen-m9/b73c6cc96e093f03 Calc!A128 `=DURATION(39448,Data!C3,Data!B9,((0.2/Data!A1)*2),2)`: Excel #NUM!, ours #VALUE!
- gen-m9/ba4ddb817a8e700e Calc!A8 `=MDURATION(39783,45383,ROUND((Data!E1-Data!$C$12),-3),1E+300,4,Data!$D12)`: Excel #NUM!, ours #DIV/0!
- gen-m9/c0681a8b150ce301 Calc!A381 `=PRICE(INDEX(Data!A5:D6,0.5),Data!$C5,0.05,(-0.05),105,2,Data!A10)`: Excel #VALUE!, ours #REF!
- gen-m9/d912671baa1364aa Calc!A303 `=PRICE(Data!D7,VALUE("Hello World"),(-1+2^-52),1E+300,1E+6,4,7)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A158
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A60
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A110
- gen-m9/3c06f9298f65a3ba.case.json (4): Calc!A113, A130, A260, A309
- gen-m9/5422bdbf173fc233.case.json (4): Calc!A146, A263, A312, A393
- gen-m9/5c6a23c4f6597f1c.case.json (3): Calc!A5, A139, A335
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A29
- gen-m9/67a4c03afd63f988.case.json (2): Calc!A176, A286
- gen-m9/79d87fc527927626.case.json (1): Calc!A219
- gen-m9/7d2979fba2c95d64.case.json (3): Calc!A58, A200, A205
- gen-m9/81bcdc0bec0c8e6b.case.json (3): Calc!A7, A108, A399
- gen-m9/89f0aee953a55a45.case.json (3): Calc!A144, A151, A392
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A158
- gen-m9/9d9ceba257f8001b.case.json (4): Calc!A108, A115, A255, A353
- gen-m9/a91bad4e84f49739.case.json (2): Calc!A77, A175
- gen-m9/abb7d565af89b849.case.json (3): Calc!A173, A190, A277
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A213
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A95
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A128
- gen-m9/ba4ddb817a8e700e.case.json (4): Calc!A8, A48, A97, A378
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A268
- gen-m9/c0681a8b150ce301.case.json (2): Calc!A87, A381
- gen-m9/d912671baa1364aa.case.json (1): Calc!A303
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A72, A112

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `YIELD(36563,45559,SUM(..)<999999999999995>,107,107,2,1)` (a huge rate) is `#NUM!` (ours 10.571526228899263) and `MDURATION(<-1.5>,46394,COVARIANCE.P(..)<#N/A>,100,2)` `#NUM!`, the settlement check before the coupon's error (ours `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/320d51765af9b6b7 Calc!A329 `=CHITEST(YIELD(36563,45559,SUM(Data!B11:B12,Data!B$8),107,107,2,1),(DAY(43015)-ODDLPRICE(35397,38233,33415,1E-300,Data!B2,109,4)))`: Excel #NUM!, ours #N/A; inner `YIELD(36563,45559,SUM(Data!B11:B12,Data!B$8),107,107,2,1)` Excel #NUM!, ours 10.571526228899263
- gen-m9/58e3253bbd3a7a82 Calc!A143 `=EXPONDIST((-(-2.5)/MONTH(6421)),MDURATION((Data!B11+Data!D4),46394,COVARIANCE.P(Data!C7,Data!C10:E10),100,2),FALSE)`: Excel #NUM!, ours #N/A; inner `MDURATION((Data!B11+Data!D4),46394,COVARIANCE.P(Data!C7,Data!C10:E10),100,2)` Excel #NUM!, ours #N/A
Cells:
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A329
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A143

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436; partly met: A143 fixed, A329 moved to T-2391, see Outcome): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2257; partly met: 46 of 50 fixed, 4 moved to T-2488, T-2496 and T-2426, see Outcome): these 50 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: for each of the four functions, an out-of-domain argument before an error argument, and an error argument before an out-of-domain one.
- [x] `make check` green; reviewers approve.

## Out of scope
- The #NUM! at a huge yield or coupon (T-2488) and YIELD's solver (T-1592, T-2391).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed the order with probes/bond-argument-check-order (298 cells, excel-graph): it is not the COUP* order. PRICE and
YIELD read and check the basis first; settlement and maturity are `#NUM!` off the calendar as they are read; DURATION
and MDURATION check the frequency as they read it and read the basis last; the date order and the rate, yld, price and
redemption checks come after every read (financial.md "PRICE, YIELD, DURATION and MDURATION check order"). `bond_args`
in financial_bonds.rs follows it; unit test `bond_functions_check_their_arguments_in_excel_order`.
Eval: gen-m9 known failures 6417 -> 6358 (59 fixed, 47 of the 52 owned), probes 2708 -> 2702 (6 fixed) plus the new
case's 298 cells passing; 0 regressions.
The 5 owned cells left are not argument order and moved to their owners:
- A huge yield, `#DIV/0!` in ours: gen-m9/3c06f9298f65a3ba A130 and ba4ddb817a8e700e A8 to T-2488.
- YIELD at a coupon rate of about 1E15: 320d51765af9b6b7 A329 to T-2391.
- An inner ODDFPRICE with its settlement past 9999-12-31: 3c06f9298f65a3ba A309 to T-2496.
- An inner `RSQ(Data!E11:E11,..)` over an empty cell, which Excel makes `#VALUE!`: 3c06f9298f65a3ba A113. That is T-2426's rule, and that task is in flight on its branch, so its file was not edited. The cell clears when T-2426 lands.
