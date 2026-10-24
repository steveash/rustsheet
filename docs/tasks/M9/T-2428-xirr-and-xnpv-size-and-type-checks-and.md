---
id: T-2428
title: "XIRR and XNPV size and type checks, and financial domain-before-text order, under the gen-m9 statistical cells (93 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
XIRR and XNPV check their arguments as Excel does: XIRR with values and dates of different counts, or a single value, is
`#NUM!` (not `#VALUE!`), an error guess is that error; XNPV with a boolean or text one-cell rate or dates is `#VALUE!` and a
number against a range of a different count `#NUM!`. The other financial functions under these cells (ODDFYIELD, ODDLYIELD,
ODDLPRICE, DURATION, PRICEMAT, PRICEDISC, RECEIVED, TBILLYIELD, TBILLPRICE, AMORLINC, AMORDEGRC, ACCRINT, DB, VDB, PV,
EFFECT, FVSCHEDULE, NPV, IRR, RATE) give Excel's error where the enclosing statistical call shows a different one.

## Context
Gap triage of the T-2255 gen-m9 sweep: in these 83 cells the innermost diverging call is a financial function inside a
statistical one (traced by evaluating every sub-expression in the engine; the enclosing function only passes the error on,
so the cell's value is the financial call's). financial.md "XIRR and XNPV under the gen-m9 statistical cells: size and
type checks (T-2255)" records the XIRR/XNPV rule (51 cells: XIRR 40, XNPV 11). The other 32 are single cells (4 of them under FREQUENCY or HYPGEOM.DIST, whose own rules then apply) whose
Excel error is inferred the same way and mostly show a domain `#NUM!` beating a text or boolean `#VALUE!`
(`ODDFYIELD(36349,-0.5,..,FALSE,4)` `#NUM!`, `PRICEMAT(..,TRUE,3)` `#NUM!`, `DB(-1E-6,286,11,1,FALSE)` `#NUM!`) or
the reverse (`EFFECT(0,9)`, `ACCRINT(..,"TRUE")`, `AMORDEGRC(..,TRUE,1)`, `ODDLYIELD(..,BINOM.DIST(..))` `#VALUE!`), plus
`PV(-1+2^-52,30,-750)` `#DIV/0!`, `TBILLPRICE(0.5,100,1E-15)` `#DIV/0!`, `FVSCHEDULE(3669,<empty cell>)` 3669 and
`IRR({-100,#N/A,60})` `#VALUE!`; confirm each with a probe of the bare call before changing its order (excel-probe skill).
T-2257 (the financial sweep) will add direct evidence; coordinate with it.
Code: `fn_xirr`, `fn_xnpv` in crates/rustsheet-engine/src/functions/financial/mod.rs and the other functions' readers.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 83 known-failure lines in eval/baseline/gen-m9.txt, 83 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A390 `=F.INV(XIRR(Data!D11:D12,Data!C6:C9,(-1)),1,1)`: Excel #NUM!, ours #VALUE!
- gen-m9/0a9226016e8f5f33 Calc!A373 `=F.INV(XIRR({0,0,-100,50,60},Data!A8:D11,((-0.0001)/Data!B12)),10,3)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/165db58d1540f014 Calc!A361 `=LOGINV(ERROR.TYPE(VALUE("x")),(123456789.123-AVERAGE(Data!E2:E11)),XNPV(1E-300,Data!D4:E12,Data!E1))`: Excel #VALUE!, ours #NUM!
- gen-m9/285d87cea1aaa73c Calc!A28 `=GROWTH((UNIQUE(Data!D3:E9,FALSE,Data!B$12)+DURATION(45412,44998,Data!$D$8,(-0.5),"2")),Data!B5:E6,Data!C10:C12,LOOKUP(HYPERLINK("#Data!A1",Data!C2),Data!A5:A12,Data!C8:C11))`: Excel #REF!, ours #NUM!
- gen-m9/40425341c5034eed Calc!A94 `=CRITBINOM(9,IRR({-100,#N/A,60}),Data!D8)`: Excel #VALUE!, ours #N/A
- gen-m9/4ab2ba98d26c8a5e Calc!A215 `=FREQUENCY(Data!B6:B11,NPER((-1),-306,5524))`: Excel #NUM!, ours #SPILL!
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A390
- gen-m9/0a9226016e8f5f33.case.json (3): Calc!A139, A231, A373
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A215
- gen-m9/165db58d1540f014.case.json (1): Calc!A361
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A71, A110
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A28
- gen-m9/320d51765af9b6b7.case.json (3): Calc!A85, A246, A346
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A305
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A295
- gen-m9/40425341c5034eed.case.json (5): Calc!A18, A94, A216, A240, A322
- gen-m9/4250f880ab640c49.case.json (1): Calc!A91
- gen-m9/4ab2ba98d26c8a5e.case.json (2): Calc!A155, A215
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A364
- gen-m9/530c636749a85209.case.json (3): Calc!A61, A199, A338
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A32
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A97, A189, A326
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A94, A142
- gen-m9/7674c89c824aca43.case.json (1): Calc!A281
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A24
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A41
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A313, A390
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A137
- gen-m9/8fec7b207f4dd753.case.json (3): Calc!A1, A8, A356
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A141, A323
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A236, A262
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A100, A186
- gen-m9/a54c738c697617c3.case.json (1): Calc!A317
- gen-m9/a5828d3925575713.case.json (1): Calc!A210
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A219
- gen-m9/a61e547a1cab994c.case.json (3): Calc!A113, A246, A353
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A23, A231, A379
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A133, A327
- gen-m9/b4173fe25109a238.case.json (1): Calc!A337
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A122
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A44
- gen-m9/bc40d37cb4d8301d.case.json (4): Calc!A63, A79, A213, A392
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A224, A373
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A188
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A28, A283
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A206
- gen-m9/cde551ea787f713a.case.json (1): Calc!A37
- gen-m9/d8266bc15533702a.case.json (1): Calc!A83
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A122
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A29, A91
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A102, A107
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A54
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A88
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A78
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A216, A285

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): XIRR and XNPV nested in the statistical calls, where Excel's `#NUM!` and ours `#VALUE!` differ.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 39 known-failure lines in eval/baseline/gen-m9.txt, 39 visibly off.
Examples:
- gen-m9/91e167cd8d7eb652 Calc!A261 `=ZTEST(Data!B2:C5,XIRR(Data!C9:E9,Data!B2:D6))`: Excel #NUM!, ours #VALUE!
- gen-m9/928d08e3e1eecb5f Calc!A54 `=VAR(Data!A9:A11,(XIRR(Data!C7,Data!B9:C9,0)+MOD((-2.5),1)))`: Excel #NUM!, ours #VALUE!
- gen-m9/a4090567758520df Calc!A148 `=VARPA(Data!B8:B9,ROUND(XIRR(3.14159,Data!A9:D10),-2))`: Excel #NUM!, ours #VALUE!
- gen-m9/afc00c8b70e78baf Calc!A160 `=SKEW.P({0,0,0},XIRR(Data!E1:E8,Data!B2:C10),Data!B7:B10)`: Excel #NUM!, ours #VALUE!
- gen-m9/eedd629f3d69fdda Calc!A145 `=VARP(-XIRR(Data!A10:A12,1e-15,Data!$C$12),Data!D6:D8)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/0072265d385d8ab2.case.json (1): Calc!A51
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A364
- gen-m9/17c110a53924beb8.case.json (1): Calc!A47
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A391
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A214
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A285
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A258
- gen-m9/46ac3ef5e61583ca.case.json (2): Calc!A103, A369
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A355
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A106
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A267
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A144
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A50
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A386
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A148
- gen-m9/91e167cd8d7eb652.case.json (2): Calc!A247, A261
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A54
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A132
- gen-m9/941c409532603d32.case.json (1): Calc!A328
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A251
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A295
- gen-m9/a4090567758520df.case.json (2): Calc!A148, A277
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A160
- gen-m9/c2684066d8df263e.case.json (1): Calc!A289
- gen-m9/c4635963fc926284.case.json (1): Calc!A71
- gen-m9/c7664e494e418395.case.json (1): Calc!A63
- gen-m9/ce312817dc9c7f0b.case.json (3): Calc!A100, A125, A267
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A292
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A360
- gen-m9/eedd629f3d69fdda.case.json (5): Calc!A15, A145, A159, A247, A394

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): XIRR and XNPV as the outer call of the financial sweep, XIRR/XNPV nested in the other financial functions, and FVSCHEDULE. Most cells are the rule above (XIRR `#NUM!` for different counts or one value, including text and booleans in the ranges; XNPV `#VALUE!` for a boolean or text one-cell rate or dates). New in these cells (docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)"): XIRR with one value and one date, both single cells or numbers, is `#N/A` (`XIRR(Data!A9:A9,Data!E9:E9)` gen-m9/30e20ba3302fea8b A110, ours `#NUM!`; `XIRR(1E15,Data!C2)` 9d9ceba257f8001b A309, ours `#VALUE!`; also with a FALSE date cell, bc04c1f478c9896c A24), while one value against several dates is `#NUM!`; an error dates or guess argument is that error before the count check (`XIRR({-100,-200,-300},PMT(..),XLOOKUP(..)<#N/A>)` is `#N/A`, 5c6a23c4f6597f1c A181); `XIRR({-100,#N/A,60},Data!$C$7)` with one FALSE date is `#VALUE!` (1cd3096e3e87c965 A48) but `#NUM!` against 5 dates (b72d952c5f52cb4d A1). FVSCHEDULE with an empty schedule cell returns the principal (`FVSCHEDULE(6527,<empty>)` = 6527, 30e20ba3302fea8b A250; ours `#VALUE!`) and `FVSCHEDULE(5916,{-1E+300,1E+300})` is 0 (89f0aee953a55a45 A349; ours `#DIV/0!`).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 246 known-failure lines in eval/baseline/gen-m9.txt, 246 visibly off.
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A48 `=XIRR({-100,#N/A,60},Data!$C$7)`: Excel #VALUE!, ours #N/A
- gen-m9/1cd3096e3e87c965 Calc!A217 `=XNPV(0.2,0.3,Data!C9:D12)`: Excel #NUM!, ours #VALUE!
- gen-m9/30e20ba3302fea8b Calc!A110 `=XIRR(Data!A9:A9,Data!E9:E9)`: Excel #N/A, ours #NUM!
- gen-m9/30e20ba3302fea8b Calc!A250 `=FVSCHEDULE(6527,Data!D1)`: Excel 6527.0, ours #VALUE!
- gen-m9/31474c20fc1dafd6 Calc!A8 `=XIRR(100,Data!D3:E4)`: Excel #NUM!, ours #VALUE!
- gen-m9/e8d55ec393460bb7 Calc!A3 `=FVSCHEDULE(2758,Data!C7)`: Excel 2758.0, ours #VALUE!
Cells:
- gen-m9/01523d247594952e.case.json (2): Calc!A36, A243
- gen-m9/0a957119475760c5.case.json (8): Calc!A47, A96, A145, A243, A340, A341, A389, A390
- gen-m9/1b60bf99d71424e7.case.json (10): Calc!A24, A29, A78, A127, A171, A176, A225, A274, A367, A372
- gen-m9/1cd3096e3e87c965.case.json (9): Calc!A48, A97, A146, A168, A217, A244, A293, A315, A391
- gen-m9/1dce4774f5c0e736.case.json (6): Calc!A22, A39, A91, A189, A238, A336
- gen-m9/30e20ba3302fea8b.case.json (6): Calc!A110, A159, A208, A250, A257, A355
- gen-m9/31474c20fc1dafd6.case.json (10): Calc!A8, A57, A106, A204, A246, A253, A295, A302, A351, A400
- gen-m9/3aa168259335be54.case.json (9): Calc!A40, A48, A146, A195, A293, A334, A342, A381, A391
- gen-m9/3c06f9298f65a3ba.case.json (5): Calc!A46, A144, A259, A291, A389
- gen-m9/5422bdbf173fc233.case.json (6): Calc!A45, A143, A184, A192, A290, A388
- gen-m9/5c6a23c4f6597f1c.case.json (9): Calc!A34, A83, A111, A132, A181, A230, A258, A328, A377
- gen-m9/5ded7594e597e31e.case.json (8): Calc!A12, A26, A75, A124, A173, A257, A271, A320
- gen-m9/6376e27be7a05871.case.json (8): Calc!A2, A51, A100, A140, A149, A198, A247, A296
- gen-m9/67a4c03afd63f988.case.json (8): Calc!A45, A94, A98, A192, A241, A290, A339, A388
- gen-m9/762cd0e4ec9a7aa9.case.json (10): Calc!A26, A75, A124, A139, A173, A222, A271, A278, A320, A369
- gen-m9/79d87fc527927626.case.json (8): Calc!A32, A81, A130, A160, A177, A179, A326, A375
- gen-m9/7d2979fba2c95d64.case.json (9): Calc!A26, A75, A173, A222, A271, A280, A320, A369, A399
- gen-m9/81bcdc0bec0c8e6b.case.json (9): Calc!A11, A60, A73, A109, A158, A220, A256, A354, A367
- gen-m9/89f0aee953a55a45.case.json (6): Calc!A36, A85, A232, A279, A330, A349
- gen-m9/9a1fa5366bc50918.case.json (11): Calc!A6, A22, A45, A94, A143, A178, A192, A241, A290, A339, A388
- gen-m9/9d9ceba257f8001b.case.json (9): Calc!A15, A64, A113, A162, A174, A211, A260, A309, A358
- gen-m9/a91bad4e84f49739.case.json (7): Calc!A49, A88, A105, A110, A252, A301, A382
- gen-m9/abb7d565af89b849.case.json (7): Calc!A39, A45, A137, A143, A241, A339, A388
- gen-m9/b6a02893a8d6b151.case.json (5): Calc!A30, A44, A275, A303, A373
- gen-m9/b72d952c5f52cb4d.case.json (7): Calc!A1, A38, A99, A148, A197, A246, A393
- gen-m9/b73c6cc96e093f03.case.json (9): Calc!A5, A25, A74, A123, A221, A270, A271, A319, A369
- gen-m9/ba4ddb817a8e700e.case.json (6): Calc!A74, A94, A143, A241, A290, A339
- gen-m9/bc04c1f478c9896c.case.json (7): Calc!A6, A24, A73, A122, A171, A269, A313
- gen-m9/c0681a8b150ce301.case.json (7): Calc!A70, A210, A217, A259, A266, A308, A315
- gen-m9/d0854366830dcdee.case.json (8): Calc!A35, A84, A133, A182, A231, A274, A329, A378
- gen-m9/d912671baa1364aa.case.json (6): Calc!A3, A37, A184, A233, A282, A380
- gen-m9/e168be7f6d3f84c2.case.json (8): Calc!A27, A39, A76, A174, A267, A272, A355, A370
- gen-m9/e8d55ec393460bb7.case.json (3): Calc!A3, A22, A185

gen-m9 engineering and database sweep (gap triage 2026-10-23, T-2258): 10 more cells nest XIRR, XNPV or PV in an engineering or database call: XIRR (`DEC2HEX(298,XIRR(Data!A9:C11,Data!A10:C11))`, `OCT2DEC(XIRR(<"?b">,..)/..)`, `BESSELJ(XIRR(..),3)`, `DELTA(..,XIRR(0,Data!C11:D12,1E-300))`) and XNPV (`DEC2OCT(DATE(..)-XNPV(2.5,{100,200,300},<empty>))`, `BESSELJ(0.5,XNPV(-2.5E+15,..))`, `GESTEP(XNPV(100,{..},<empty>)+..)`) are `#NUM!`; `HEX2BIN(HLOOKUP(XNPV(-1+2^-52,<"#N/A" text>,{..}),..),1)` and `COMPLEX(XNPV(7,<TRUE cell>,..)/..,..)` `#VALUE!`; `DSTDEVP(Db!A1:D1,PV(-1+2^-52,340,-186,643),Data!A9:A9)` `#DIV/0!` (PV's `#DIV/0!`, as this task's `PV(-1+2^-52,30,-750)`).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 10 known-failure lines in eval/baseline/gen-m9.txt, 10 formulas, 10 visibly off.
Examples:
- gen-m9/4fffa5addab1f8af Calc!A325 `=BESSELJ(0.5,XNPV(((-2.5)*1000000000000000.0),Data!D5:D7,Data!A5:D6))`: Excel #NUM!, ours #VALUE!
- gen-m9/5bdef392d16a5efb Calc!A67 `=DEC2HEX(298,XIRR(Data!A9:C11,Data!A10:C11))`: Excel #NUM!, ours #VALUE!
- gen-m9/7820ebc42c7a56a6 Calc!A85 `=COMPLEX((XNPV(7,Data!E5,Data!A3:D10)/Data!$C9),(IF(TRUE,Data!B$8,Data!B$8)/RECEIVED((-1),44734,832748,1E-300,3)))`: Excel #VALUE!, ours #NUM!
- gen-m9/7820ebc42c7a56a6 Calc!A249 `=OCT2DEC((XIRR(Data!A10:B10,{39448;39508;39859;39904;40000},2)/(2.5/Data!D8)))`: Excel #NUM!, ours #VALUE!
- gen-m9/8cbfd5e99d2e82da Calc!A140 `=BESSELJ(XIRR(SUM(Data!E7:E7,Data!C6:C7),{39448;39508;39859;39904;40000},(0.2^Data!D9)),3)`: Excel #NUM!, ours #VALUE!
- gen-m9/8fa8b572ebc1f001 Calc!A4 `=DEC2OCT((DATE(1908,Data!E8,22)-XNPV(2.5,{100,200,300},Data!A2)))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A325
- gen-m9/5bdef392d16a5efb.case.json (1): Calc!A67
- gen-m9/7820ebc42c7a56a6.case.json (2): Calc!A85, A249
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A140
- gen-m9/8fa8b572ebc1f001.case.json (1): Calc!A4
- gen-m9/c2d778463fe008ed.case.json (1): Calc!A208
- gen-m9/d9d55972b50e55e8.case.json (1): Calc!A21
- gen-m9/e882060cf4613302.case.json (1): Calc!A257
- gen-m9/f61a73cd9af21e6d.case.json (1): Calc!A275

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `XIRR({-100,TRUE,60},Data!A7:A8,1E-9)` (three values, two dates) is `#NUM!` (ours `#VALUE!`) and `ACCRINT(39448,..,100,INDEX(Data!C2:E12,42,100)<#REF!>,836,4,2,2)` `#NUM!`, the date checks before the rate's error (ours `#REF!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/8dd2312c96effb1b Calc!A381 `=BINOM.DIST(XIRR({-100,TRUE,60},Data!A7:A8,1E-9),4,1E-300,ISBLANK(CEILING((-2.5),Data!B10)))`: Excel #NUM!, ours #VALUE!; inner `XIRR({-100,TRUE,60},Data!A7:A8,1E-9)` Excel #NUM!, ours #VALUE!
- gen-m9/ecc87f80077959e0 Calc!A234 `=AVERAGEA(Data!C7:C9,Data!B2:B8,ACCRINT(39448,AVERAGE(Data!A2:A6,Data!B6),(Data!A7+100),INDEX(Data!C2:E12,Data!$E6,100),836,4,2,2))`: Excel #NUM!, ours #REF!; inner `ACCRINT(39448,AVERAGE(Data!A2:A6,Data!B6),(Data!A7+100),INDEX(Data!C2:E12,Data!$E6,100),836,4,2,2)` Excel #NUM!, ours #REF!
Cells:
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A381
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A234

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2258): these 10 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): the 246 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 122 known-failure lines (83 from T-2255, 39 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for the XIRR/XNPV shapes above (different counts, one value, an error guess, a boolean dates cell).
- [ ] financial.md's entry gets an "Ours:" line naming this task; each of the 32 other cells is fixed or listed in the
      Outcome with its probe result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR/XNPV values (their solver tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Progress
2026-10-24 (opus run): XIRR's size checks landed on this branch (`xirr_size_error` in functions/financial/mod.rs; order in
financial.md "XIRR and XNPV under the gen-m9 statistical cells"). Eval: gen-m9 known failures -192, probes -6,
0 regressions. Still open: the XNPV rules (boolean or text one-cell rate or dates `#VALUE!`, a number against a range of a
different count `#NUM!`), FVSCHEDULE's empty schedule cell, and the other financial functions' domain-before-text orders.

2026-10-24, later (same run, out of session budget; work in progress, not mergeable yet). Round-1 reviews: security
REQUEST_CHANGES (the old `holds_text_or_boolean_cell` scanned an untrimmed whole-column range); quality REQUEST_CHANGES
(duplicated used-area trimming, now one helper `xirr_range_end`; add exact-value unit tests in
test/test_fn_financial_lift.rs); excel-fidelity REQUEST_CHANGES ("fewer values than dates with a text value is `#VALUE!`"
is contradicted by ~15 direct cells, e.g. gen-m9/0a957119475760c5 A47; narrowed to a single value cell, which removed the
scan); architecture APPROVE. The narrowed rule fixes 31 more gen-m9 cells but regresses 4 against the committed
baseline: gen-m9/1dce4774f5c0e736 A91, 89f0aee953a55a45 A36, ba4ddb817a8e700e A339, c7664e494e418395 A204 (the last is
masked by LCM's `#VALUE!` inside ZTEST, per the fidelity review). Next: trace those 4 (`rustsheet-eval show`), fix,
`make eval-baseline`, add the unit tests, re-run all four reviewers.
