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
