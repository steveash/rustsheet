---
id: T-2611
title: "XIRR's error-guess order and the other financial functions' domain-before-text orders under the gen-m9 cells (86 cells left by T-2428)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
XIRR, XNPV and the financial functions nested in the gen-m9 statistical, financial and engineering cells give Excel's
error where the cells T-2428 left open still differ: chiefly XIRR beside an error or out-of-domain guess, and the
domain `#NUM!` (or `#DIV/0!`) that Excel returns before a text or boolean `#VALUE!` in ODDFYIELD, ODDLPRICE, ODDLYIELD,
AMORLINC, AMORDEGRC, RECEIVED, TBILLYIELD, TBILLPRICE, PRICEMAT, PRICEDISC, DB, VDB, PV, YIELD, ACCRINT and DURATION.

## Context
Split from T-2428 (2026-10-24), which landed XIRR's size checks, XNPV's rate/one-cell/size order, FVSCHEDULE's empty
schedule cell and Z.TEST's direct-error data (financial.md "XIRR and XNPV under the gen-m9 statistical cells";
statistics.md "Z.TEST: a direct error data value comes first"). These 86 owned cells of T-2428 remain in
eval/baseline/gen-m9.txt; each needs its inner call traced (`rustsheet-eval show`, then a probe of the bare call with the
excel-probe skill before changing an order). Groups seen while splitting:
- XIRR with an error guess: Excel gives the guess's error beside numeric values (`XIRR(Data!A11:C12,AVERAGE(..),VAR.P(<1 cell>))`
  `#DIV/0!`, 67a4c03afd63f988 A388; `XIRR((-1000, 300, 400, 500),Data!B9:E12,INT(FORECAST.LINEAR(..)))` `#N/A`, 1b60bf99d71424e7 A29)
  but `#VALUE!` beside text values (b72d952c5f52cb4d A50); ours keeps `#VALUE!` for any non-number guess (`fn_xirr`'s
  `number_guess`). Also an out-of-domain number guess (`1E+10`, `-0.9999999999`) beside mismatched counts is `#NUM!`.
- XIRR with a computed one-number dates argument (`XIRR(<range>,SUM(..))`, `XIRR(<n>,RANK.EQ(..))`): Excel `#N/A` or `#NUM!`, ours `#VALUE!`.
- XNPV with equal counts and a text/boolean in both ranges (1b60bf99d71424e7 A171 `#VALUE!`) or a boolean date in a range
  (5ded7594e597e31e A12 `#NUM!`): the element reading order of values and dates is unprobed.
- The other financial functions' domain-before-text orders (examples in T-2428's Context: `ODDFYIELD(36349,-0.5,..,FALSE,4)`
  `#NUM!`, `PRICEMAT(..,TRUE,3)` `#NUM!`, `DB(-1E-6,286,11,1,FALSE)` `#NUM!`, `EFFECT(0,9)`, `ACCRINT(..,"TRUE")` `#VALUE!`),
  `PV(-1+2^-52,30,-750)` `#DIV/0!` (ours `#NUM!`), `FVSCHEDULE(5916,{-1E+300,1E+300})` 0 (ours `#DIV/0!`), FREQUENCY over a
  scalar error bins argument (`#NUM!`, ours `#SPILL!`).
Unprobed corners T-2428's review raised (probe each, then follow or record): whether the size checks compare a whole
column's full count or its used area (`XIRR(A:A,B1:B10)`, `XNPV(r,Sheet1!A:A,Sheet2!B:B)`; ours trims to the used area,
`flow_range_end`); a literal boolean rate `XNPV(TRUE,..)` (ours `#VALUE!` as for a cell); and whether XIRR puts an error
dates argument before the values' cells as XNPV does (ours reads the values first).
Some cells may be decided by an enclosing function instead (FORECAST.ETS is `#N/IMPL!`, TRIMMEAN); move those to their
owners.
Code: `fn_xirr`, `xirr_size_error`, `fn_xnpv`, `xnpv_size_error` in crates/rustsheet-engine/src/functions/financial/mod.rs
and the other functions' readers.

Owned gen-m9 cells (left by T-2428, excel-graph, 2026-10-24): 86 known-failure lines in eval/baseline/gen-m9.txt.
Cells:
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A231, A373
- gen-m9/0a957119475760c5.case.json (2): Calc!A243, A340
- gen-m9/1b60bf99d71424e7.case.json (2): Calc!A29, A171
- gen-m9/1cd3096e3e87c965.case.json (2): Calc!A244, A293
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A91, A189
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A110
- gen-m9/30e20ba3302fea8b.case.json (3): Calc!A159, A208, A257
- gen-m9/31474c20fc1dafd6.case.json (2): Calc!A253, A351
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A246
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A46
- gen-m9/40425341c5034eed.case.json (3): Calc!A18, A216, A322
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A215
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A364
- gen-m9/530c636749a85209.case.json (2): Calc!A61, A338
- gen-m9/5422bdbf173fc233.case.json (2): Calc!A192, A290
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A267
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A97, A189
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A144
- gen-m9/5c6a23c4f6597f1c.case.json (3): Calc!A34, A83, A181
- gen-m9/5ded7594e597e31e.case.json (2): Calc!A12, A271
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A94
- gen-m9/6376e27be7a05871.case.json (1): Calc!A296
- gen-m9/67a4c03afd63f988.case.json (3): Calc!A241, A290, A388
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A50
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A24
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A222
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A158, A354
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A148
- gen-m9/89f0aee953a55a45.case.json (3): Calc!A36, A330, A349
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A137
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A1
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A141
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A143
- gen-m9/a4090567758520df.case.json (1): Calc!A277
- gen-m9/a54c738c697617c3.case.json (1): Calc!A317
- gen-m9/a5828d3925575713.case.json (1): Calc!A210
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A353
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A23, A231
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A301
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A275
- gen-m9/b72d952c5f52cb4d.case.json (3): Calc!A38, A197, A393
- gen-m9/ba4ddb817a8e700e.case.json (2): Calc!A290, A339
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A213
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A224
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A266
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A28
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A206
- gen-m9/cde551ea787f713a.case.json (1): Calc!A37
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A267
- gen-m9/d0854366830dcdee.case.json (3): Calc!A84, A133, A182
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A292
- gen-m9/d912671baa1364aa.case.json (1): Calc!A282
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A355
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A91, A234
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A145
- gen-m9/f61a73cd9af21e6d.case.json (1): Calc!A275
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A216, A285

Fixed by T-2483 (2026-10-24; financial.md "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order"): 2801c1c11c21e134
A110 (an inner ODDFYIELD whose issue date, 123456789.123 times a cell, is past 9999-12-31: `#NUM!` as it is read) and 320d51765af9b6b7 A246 (an inner ODDLPRICE whose
settlement past 9999-12-31 is `#NUM!` as it is read); both lines are gone from the baseline.

## Acceptance criteria
- [ ] The 86 known-failure lines listed above disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), or each one
      left is listed in the Outcome with its traced cause and owner task; no new known-failure lines in any baseline.
- [ ] Every order changed is backed by a probe of the bare call and recorded in docs/excel-semantics/financial.md.
- [ ] Unit tests for each order changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR/XNPV values (their solver tasks).

## Cells added by the Phase 1 triage (T-0090, 2026-10-27)
From the parked T-2730: XIRR gen-m9/1dce4774f5c0e736 Calc!A91 `XIRR(Data!D10:D10,Data!B7:E8)`, #NUM! expected, #VALUE! today (an error-order cell).

## Progress (2026-10-30, opus run; resume from here)
Traced at 09f899b9 (after T-3268). 55 of the 86 owned lines are still in eval/baseline/gen-m9.txt; the other 31 pass now.
The 55 by outermost function: XIRR 37, XNPV 2, LOGNORM.INV 2, and one each of ZTEST, WEIBULL, STDEVPA, PERCENTRANK.INC, PEARSON,
ODDLYIELD, LOGNORMDIST, HYPGEOM.DIST, GAMMA.INV, FVSCHEDULE, FINV, F.INV, EXPONDIST and BETADIST.
The 37 top-level XIRR cells are bare calls, so each one is direct evidence. Ours is `#VALUE!` on 35 of them, and `#N/A` on
30e20ba3302fea8b A208 and 67a4c03afd63f988 A290. `rustsheet-eval show` gives expected / ours:
- Calc!A243 =XIRR(Data!D2:D6,TINV(MINUTE(0.5),10000000001),DAY(1)) -> #NUM! (ours #VALUE!)
- 0a957119475760c5 Calc!A243 =XIRR(Data!D2:D6,TINV(MINUTE(0.5),10000000001),DAY(1)) -> #NUM! (ours #VALUE!)
- 1b60bf99d71424e7 Calc!A29 =XIRR({-1000,300,400,500},Data!B9:E12,INT(FORECAST.LINEAR(42,123456789.123,Data!A1:B5))) -> #N/A (ours #VALUE!)
- 1cd3096e3e87c965 Calc!A244 =XIRR(Data!A5:C12,SIGN(F.TEST(Data!A2,Data!A4:C5))) -> #DIV/0! (ours #VALUE!)
- 1cd3096e3e87c965 Calc!A293 =XIRR((FILTER(Data!C1:D12,(Data!B1:B12<>(-0.5)),Data!A9:D9)/FDIST(Data!A6,8,Data!$A7)),Data!C1:E10,MATCH(Data!A$4,Data!B1:D3,-1)) -> #N/A (ours #VALUE!)
- 1dce4774f5c0e736 Calc!A91 =XIRR(Data!D10:D10,Data!B7:E8) -> #NUM! (ours #VALUE!)
- 1dce4774f5c0e736 Calc!A189 =XIRR((IMAGINARY("1-2i")+ODD(1E+21)),(PRODUCT(Data!D3:D10,Data!A10:A10)+Data!E8),ODDLPRICE(36295,(0.1-Data!E$3),YEARFRAC(8606,8673),1E-300,(Data!E3*Data!$D3),102,1)) -> #NUM! (ours #VALUE!)
- 30e20ba3302fea8b Calc!A159 =XIRR(Data!C2:D11,Data!B9:E9,GAMMADIST(1000000000000000.0,SUM(Data!D4),BESSELY(6,4),AND(Data!E7:E9,Data!A7:A11))) -> #NUM! (ours #VALUE!)
- 30e20ba3302fea8b Calc!A208 =XIRR((Data!E$2-NORMSDIST(Data!C3)),WEIBULL.DIST((Data!$C9/Data!A12),1.5E+10,99999,(Data!B10=Data!$B8))) -> #NUM! (ours #N/A)
- 30e20ba3302fea8b Calc!A257 =XIRR(Data!A7:B11,LOGNORMDIST((Data!C8/(-1)),1E-300,FINV(Data!C2,7,6)),T.TEST(Data!A1:D7,Data!A4:B4,TOCOL(Data!A5:A5),3)) -> #NUM! (ours #VALUE!)
- 31474c20fc1dafd6 Calc!A253 =XIRR(DEGREES(0.5),{39448,39448,39448,39448,39448},(RANK(100,Data!A7:D7)-Data!E6)) -> #N/A (ours #VALUE!)
- 31474c20fc1dafd6 Calc!A351 =XIRR(Data!C9:C11,{39448,39508,39859},0.1) -> #NUM! (ours #VALUE!)
- 3c06f9298f65a3ba Calc!A46 =XIRR(Data!A10:D10,HYPGEOM.DIST(0,4,(-1),VARA(Data!E5,{TRUE,FALSE,1}),(Data!D$2>Data!E4)),(-0.9999999999)) -> #NUM! (ours #VALUE!)
- 5422bdbf173fc233 Calc!A192 =XIRR(Data!B5:E11,TBILLPRICE(36586,47664,0.5),1E-10) -> #NUM! (ours #VALUE!)
- 5422bdbf173fc233 Calc!A290 =XIRR((Data!C5+SUMIF(Data!B7:C10,1,Data!A2:D5)),Data!A5:D8,SECOND((Data!E9*0.2))) -> #NUM! (ours #VALUE!)
- 5c6a23c4f6597f1c Calc!A83 =XIRR(Data!A1:D8,(POWER(59,Data!E10)/RANK.AVG(Data!E$2,Data!A4:B12,Data!A2)),10) -> #N/A (ours #VALUE!)
- 5c6a23c4f6597f1c Calc!A181 =XIRR({-100,-200,-300},PMT(1E-9,180,1,(-0.5)),XLOOKUP(0.5,Data!B10:B10,Data!C7:C8)) -> #N/A (ours #VALUE!)
- 5ded7594e597e31e Calc!A271 =XIRR({-100,TRUE,60},Data!D1:D8,((-1)/SINH(1E+8))) -> #NUM! (ours #VALUE!)
- 6376e27be7a05871 Calc!A296 =XIRR(Data!A10:D12,COVARIANCE.P(Data!A2:C10,0)) -> #N/A (ours #VALUE!)
- 67a4c03afd63f988 Calc!A241 =XIRR(Data!B11:E11,LOOKUP("?b",{1;#N/A;3},Data!D7:D7)) -> #N/A (ours #VALUE!)
- 67a4c03afd63f988 Calc!A290 =XIRR({-100,#N/A,60},(PRICEMAT(39090,44566,33662,0,(-1),4)/1e-15)) -> #NUM! (ours #N/A)
- 67a4c03afd63f988 Calc!A388 =XIRR(Data!A11:C12,AVERAGE(Data!C10:C10),VAR.P(Data!D6:D6)) -> #DIV/0! (ours #VALUE!)
- 7d2979fba2c95d64 Calc!A222 =XIRR((((-2.5)/(-2.5))+42),RANK.EQ(Data!$D7,Data!A8:B8,0)) -> #N/A (ours #VALUE!)
- 81bcdc0bec0c8e6b Calc!A158 =XIRR(Data!E$4,OFFSET(Data!C5:D5,Data!A2,1,(0.3-Data!C10)),7) -> #REF! (ours #VALUE!)
- 81bcdc0bec0c8e6b Calc!A354 =XIRR(Data!B2:E4,{45322,45351,45352,45657,45658},-CSC(134217728)) -> #NUM! (ours #VALUE!)
- 89f0aee953a55a45 Calc!A36 =XIRR(Data!C7:C7,Data!A4:C7,1000000000000000.0) -> #NUM! (ours #VALUE!)
- 89f0aee953a55a45 Calc!A330 =XIRR(Data!B6:C11,Data!A10,0.5) -> #NUM! (ours #VALUE!)
- 9a1fa5366bc50918 Calc!A143 =XIRR(Data!C3:E5,Data!C4:E7,ATANH(FV(Data!D7,215,-9,-328))) -> #NUM! (ours #VALUE!)
- a91bad4e84f49739 Calc!A301 =XIRR((0.2-(-0.0001)%),Data!A4:C9,CHITEST(Data!$E4,0.5)) -> #N/A (ours #VALUE!)
- b6a02893a8d6b151 Calc!A275 =XIRR(Data!D9:E10,DECIMAL(REPT(Data!A4,5),36),1E+10) -> #NUM! (ours #VALUE!)
- b72d952c5f52cb4d Calc!A393 =XIRR((2-(Data!C$9-Data!A9)),TIME(-Data!E2,2,((-0.0001)-123456789.123))) -> #NUM! (ours #VALUE!)
- ba4ddb817a8e700e Calc!A290 =XIRR(Data!B10:D10,SUM(Data!D1:D1,AVERAGE(0.2,Data!C10),(2/Data!C5))) -> #DIV/0! (ours #VALUE!)
- ba4ddb817a8e700e Calc!A339 =XIRR(Data!C11,Data!C6:D11,VAR.P(Data!B1:B5)) -> #NUM! (ours #VALUE!)
- c0681a8b150ce301 Calc!A266 =XIRR(Data!A6:B12,Data!C$7) -> #NUM! (ours #VALUE!)
- d0854366830dcdee Calc!A84 =XIRR(MIN(Data!D3:D10,Data!C1,WEIBULL(123456789.123,2.5,Data!E1,Data!$A5)),(LCM(Data!E4,Data!$D$1)*DMIN(Db!A1:D1,2,Db!F1:G2))) -> #NUM! (ours #VALUE!)
- d0854366830dcdee Calc!A133 =XIRR(Data!A9:B12,Data!C8,(DAY(6059)*(7*2.5))) -> #NUM! (ours #VALUE!)
- d0854366830dcdee Calc!A182 =XIRR({-1000,300,400,500},Data!A3:C8,DECIMAL(("TRUE"&"ü"),36)) -> #NUM! (ours #VALUE!)
- d912671baa1364aa Calc!A282 =XIRR(Data!$E3,NORM.DIST(SUMPRODUCT(Data!$E5,(-0.0001)),SUMXMY2(Data!D3:E7,Data!A5:B5),1.5,(-1)),(((-1)/Data!D7)-BINOM.DIST.RANGE(0.5,1E-10,0))) -> #N/A (ours #VALUE!)
Next step: read each case's Data cells against financial.md "Order (XIRR, T-2428)", which predicts our `#VALUE!` for most of
these cells, so that order needs revising from them. Start with the cells where Excel returns the guess's or the dates'
error rather than `#VALUE!`: 1b60bf99d71424e7 A29, 1cd3096e3e87c965 A244 and A293, 5c6a23c4f6597f1c A83 and A181, 6376e27be7a05871 A296, 67a4c03afd63f988 A241 and A388,
a91bad4e84f49739 A301, d912671baa1364aa A282, ba4ddb817a8e700e A290 and 81bcdc0bec0c8e6b A158. Then do the `#NUM!` cells with an out-of-domain number guess or a
non-date dates argument.

Partial fix on this branch (not reviewed yet; `make check` not run): XIRR returns an error values argument first, then an
error dates argument, before it reads the values' cells (financial.md, T-2611 sentence in "Order (XIRR, T-2428)").
`rustsheet-eval check --suite gen-m9`: 3094 → 3074 known failures, 20 fixed, 0 regressions, all 20 owned by this task:
0a957119475760c5 A243, 1cd3096e3e87c965 A244, 30e20ba3302fea8b A257, 3c06f9298f65a3ba A46, 40425341c5034eed A216, 5422bdbf173fc233 A192, 583ca2baea0bb30a A267, 5c6a23c4f6597f1c A83, 6376e27be7a05871 A296, 67a4c03afd63f988 A241, 67a4c03afd63f988 A290, 67a4c03afd63f988 A388, 7d2979fba2c95d64 A222, 81bcdc0bec0c8e6b A158, b6a02893a8d6b151 A275, b72d952c5f52cb4d A393, ba4ddb817a8e700e A290, ce312817dc9c7f0b A267, d0854366830dcdee A84, d912671baa1364aa A282.
A first try that put the dates error before an error values argument regressed 79d87fc527927626 A228, 81bcdc0bec0c8e6b
A305 and bc04c1f478c9896c A220. 35 owned lines remain. Next: run `make check` and the review panel on this commit, then
continue with the guess-error and `#NUM!` groups above.

Resumed 2026-10-30 (opus run, work-loop resume of an idle branch).
