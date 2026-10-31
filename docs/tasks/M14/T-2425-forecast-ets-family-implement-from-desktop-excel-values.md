---
id: T-2425
title: "FORECAST.ETS family: implement from desktop Excel values; owns the 877 gen-m9 cells excel-graph leaves #N/A"
milestone: M14
lane: functions-stat
status: blocked
priority: P3
size: M
tier: opus
depends_on: [T-1553, T-2424]
campaign: visible
---

## Goal
FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT compute desktop Excel's values (the
AAA exponential-smoothing forecast, its confidence interval, the detected seasonality and the fit statistics), and these
877 gen-m9 cells, `#N/A` on excel-graph because Excel for the web never computes the family, are scored against desktop
values.

## Context
Gap triage of the T-2255 gen-m9 sweep. These cells have no argument error, text or lift that decides them (that step is
T-2424): excel-graph gives `#N/A` for every one (statistics.md "FORECAST.ETS family: excel-graph never computes it",
T-1293), including 53 cells where the call is nested in another function whose value then is `#N/A`. A validation-only
engine that returns `#N/A` would pass them on the web and be wrong on desktop, which T-1293's Outcome rules out, so they
cannot be fixed against the excel-graph sidecars. They need desktop values (T-1553, the `excel-desktop` backend, decision
0039; decision 0030: desktop evidence wins where it exists): re-fill these cases (or a desktop sidecar of them) on desktop
Excel, then spec the algorithm (docs/specs/, a task of its own if it is L) and implement it in
crates/rustsheet-engine/src/functions/statistical/forecast.rs. Until then the cells stay known failures, owned here so that
none is unowned (decision 0083).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 877 known-failure lines in eval/baseline/gen-m9.txt, 877 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A275 `=LOGINV(ROUND(COUNT(Data!B7:B8,1e-15,"ABC"),3),IF((Data!$B12>Data!C6),FORECAST.ETS.CONFINT((-1),Data!B6:B11,Data!A5:D6),Data!D5),(HLOOKUP("x~*",Data!C10:C10,1)*(0.5*3.14159)))`: Excel #N/A, ours #N/IMPL!
- gen-m9/165db58d1540f014 Calc!A233 `=FTEST(Data!B5:E6,F.INV(FORECAST.ETS.CONFINT(100,Data!B4:D12,0.2,0.3,2,0,5),2,1))`: Excel #N/A, ours #VALUE!
- gen-m9/22656c225ff4f22a Calc!A381 `=FORECAST.ETS(((-0.5)-(Data!$C$4+Data!B$4)),Data!C8:D12,YIELDDISC(37374,FORECAST.ETS.CONFINT(0.2,123456789.123,0,Data!D$10,3,0),83,103),0,1)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/22656c225ff4f22a Calc!A41 `=FORECAST.ETS(DATEDIF(45283,367,HSTACK(Data!A10:D11)),MAX(Data!C7:C8,Data!$E8,Data!C11:C11),(FORECAST.ETS.CONFINT(3.14159,(-0.5),Data!B11:C12,3.14159)+(Data!$E11*100)),1)`: Excel #SPILL!, ours #N/IMPL!
- gen-m9/ee2497a5e683c5d9 Calc!A312 `=CONFIDENCE.NORM(NORMDIST(-Data!B9,TINV(1.5,9),10000000001,IFNA(#N/A,2.5)),(ROUND(Data!C11,Data!C7)-FORECAST.ETS.STAT(Data!D10,7,7,1,0)),COUNTIFS(Data!B6:D10,"?b?"))`: Excel #N/A, ours #NUM!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A275, A390
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A173, A347
- gen-m9/0a9226016e8f5f33.case.json (3): Calc!A210, A263, A372
- gen-m9/165db58d1540f014.case.json (3): Calc!A210, A233, A306
- gen-m9/22656c225ff4f22a.case.json (160): Calc!A4, A7, A15, A16, A19, A20, A21, A23, A26, A27, A33, A34, A39, A41, A43, A45, A47, A48, A49, A51, A52, A53, A56, A62, A63, A64, A66, A68, A71, A75, A77, A79, A80, A81, A84, A87, A92, A96, A97, A99, A101, A103, A106, A107, A108, A109, A112, A115, A117, A120, A125, A127, A128, A129, A135, A140, A143, A144, A145, A149, A151, A152, A153, A159, A160, A163, A164, A168, A176, A178, A179, A182, A187, A188, A191, A199, A206, A208, A210, A211, A212, A216, A219, A225, A226, A227, A228, A229, A230, A233, A234, A236, A237, A239, A241, A242, A244, A248, A251, A252, A254, A255, A262, A267, A270, A271, A272, A275, A278, A279, A282, A283, A284, A288, A292, A296, A298, A304, A307, A309, A314, A315, A316, A317, A319, A320, A323, A324, A326, A332, A335, A336, A340, A341, A343, A345, A348, A349, A356, A359, A360, A361, A364, A365, A366, A368, A369, A374, A379, A380, A381, A383, A386, A387, A388, A393, A395, A397, A399, A400
- gen-m9/27601819397dd56e.case.json (1): Calc!A334
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A22
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A126
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A265
- gen-m9/4250f880ab640c49.case.json (2): Calc!A324, A352
- gen-m9/43a13a091907b402.case.json (180): Calc!A7, A10, A11, A13, A15, A17, A22, A23, A24, A26, A27, A29, A30, A33, A34, A35, A36, A38, A39, A42, A45, A49, A50, A51, A52, A54, A55, A58, A59, A62, A63, A64, A66, A70, A71, A72, A74, A75, A78, A79, A82, A86, A87, A88, A90, A92, A94, A95, A96, A99, A102, A106, A108, A110, A111, A113, A115, A120, A121, A126, A130, A133, A134, A135, A138, A139, A142, A146, A147, A149, A150, A151, A152, A154, A155, A158, A159, A164, A166, A167, A171, A173, A174, A175, A177, A179, A182, A183, A187, A188, A192, A194, A202, A208, A212, A213, A214, A218, A219, A225, A226, A227, A228, A235, A242, A243, A244, A249, A251, A254, A255, A258, A259, A260, A261, A262, A264, A265, A266, A269, A270, A271, A275, A278, A280, A281, A282, A286, A287, A290, A294, A295, A298, A299, A300, A303, A304, A305, A306, A307, A309, A311, A320, A323, A325, A326, A327, A329, A330, A335, A338, A339, A342, A344, A348, A350, A351, A353, A354, A357, A358, A362, A363, A364, A366, A367, A370, A373, A374, A375, A381, A382, A383, A384, A386, A391, A393, A395, A396, A397
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A329
- gen-m9/55fc8c4ffbd7c7cc.case.json (1): Calc!A8
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A329
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A153
- gen-m9/623663cacea53bbc.case.json (2): Calc!A160, A169
- gen-m9/67f5e0fe790e6bec.case.json (155): Calc!A4, A8, A10, A11, A15, A18, A20, A22, A23, A24, A27, A28, A34, A35, A41, A44, A52, A55, A56, A58, A59, A62, A66, A67, A71, A72, A78, A82, A83, A86, A87, A90, A92, A93, A94, A95, A98, A99, A102, A106, A107, A108, A109, A110, A111, A114, A119, A121, A131, A134, A138, A139, A140, A144, A146, A147, A148, A151, A155, A156, A159, A161, A164, A166, A167, A168, A170, A171, A175, A179, A181, A182, A184, A186, A189, A190, A191, A197, A199, A200, A203, A207, A208, A213, A214, A218, A219, A220, A222, A225, A230, A235, A239, A240, A242, A243, A246, A252, A254, A257, A263, A266, A271, A275, A276, A278, A281, A286, A287, A290, A294, A295, A297, A299, A300, A302, A303, A310, A311, A312, A314, A315, A321, A326, A329, A331, A333, A339, A340, A343, A350, A351, A352, A354, A355, A358, A361, A362, A363, A364, A365, A367, A370, A371, A373, A381, A384, A386, A388, A390, A391, A394, A395, A396, A398
- gen-m9/7674c89c824aca43.case.json (2): Calc!A261, A378
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A50
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A272
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A342
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A240
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A61
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A221
- gen-m9/a7bf0d8d81d33ba6.case.json (3): Calc!A44, A57, A64
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A392
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A69
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A14, A138
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A164
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A201
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A312, A358
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A116, A309
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A187
- gen-m9/cde551ea787f713a.case.json (1): Calc!A280
- gen-m9/d8266bc15533702a.case.json (2): Calc!A76, A369
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A183, A293
- gen-m9/ec2a1d9194a84396.case.json (184): Calc!A2, A4, A8, A11, A12, A14, A15, A16, A17, A21, A22, A24, A26, A32, A34, A35, A36, A40, A42, A44, A45, A46, A47, A49, A50, A52, A53, A54, A55, A58, A61, A68, A69, A72, A76, A78, A79, A81, A82, A88, A90, A92, A94, A95, A96, A98, A100, A102, A103, A104, A105, A109, A111, A112, A113, A114, A115, A116, A117, A118, A122, A123, A124, A125, A126, A127, A128, A129, A130, A132, A134, A135, A142, A143, A145, A150, A152, A158, A160, A166, A168, A169, A170, A171, A172, A175, A176, A180, A181, A182, A186, A187, A188, A192, A194, A197, A198, A200, A202, A207, A210, A212, A213, A214, A216, A219, A220, A221, A223, A224, A226, A228, A231, A235, A238, A242, A244, A249, A250, A251, A252, A254, A255, A256, A258, A264, A265, A266, A271, A273, A274, A276, A278, A280, A282, A284, A286, A288, A290, A293, A294, A296, A300, A306, A308, A313, A314, A318, A320, A328, A331, A334, A335, A338, A339, A340, A341, A342, A346, A351, A354, A355, A357, A358, A359, A364, A365, A366, A367, A372, A373, A374, A375, A376, A378, A380, A384, A385, A386, A390, A394, A396, A398, A400
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A8, A312
- gen-m9/f2bd5f380ec002bf.case.json (2): Calc!A214, A258
- gen-m9/fa25422d6f18c157.case.json (145): Calc!A6, A8, A10, A11, A12, A14, A16, A20, A21, A22, A32, A35, A37, A38, A42, A44, A45, A53, A56, A60, A62, A65, A67, A68, A70, A71, A74, A79, A80, A88, A89, A90, A91, A96, A97, A98, A99, A102, A104, A106, A109, A114, A116, A119, A128, A130, A131, A137, A138, A142, A144, A146, A148, A149, A151, A156, A163, A165, A166, A172, A173, A176, A177, A183, A185, A186, A187, A188, A189, A192, A193, A199, A200, A202, A207, A213, A215, A216, A218, A224, A227, A228, A230, A235, A244, A246, A247, A248, A251, A252, A254, A255, A256, A257, A258, A264, A267, A268, A269, A272, A273, A275, A276, A277, A280, A282, A284, A285, A288, A297, A298, A299, A301, A304, A306, A308, A313, A316, A320, A325, A328, A332, A336, A337, A338, A340, A341, A342, A349, A353, A370, A371, A372, A376, A377, A378, A380, A381, A382, A384, A390, A394, A395, A399, A400

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): FORECAST.ETS* calls nested in another function with no error argument of their own; excel-graph's `#N/A` passes through the outer call.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 63 known-failure lines in eval/baseline/gen-m9.txt, 63 visibly off.
Examples:
- gen-m9/0bbab6206b5c9a07 Calc!A359 `=PERCENTRANK(5,FORECAST.ETS((-1),(-1),Data!D9:D11))`: Excel #N/A, ours #N/IMPL!
- gen-m9/73f053591a54bc43 Calc!A30 `=STDEV.P(FORECAST.ETS.STAT((0.5+0.2),0.3,3),Data!D7:D8)`: Excel #N/A, ours #N/IMPL!
- gen-m9/7980ffb6095a340d Calc!A253 `=QUARTILE.INC((0/FORECAST.ETS.SEASONALITY(Data!B5,100)),Data!$C1)`: Excel #N/A, ours #N/IMPL!
- gen-m9/9bc247733a1ca9fa Calc!A10 `=T.DIST.2T(FORECAST.ETS(Data!C$3,LET(w,Data!D4,w),2.5,3,1,7),9)`: Excel #N/A, ours #N/IMPL!
- gen-m9/d37313406b9f6e55 Calc!A40 `=PERCENTRANK.EXC(Data!A7:D11,FORECAST.ETS.STAT(Data!A5:D6,7,5,2))`: Excel #N/A, ours #N/IMPL!
Cells:
- gen-m9/0072265d385d8ab2.case.json (1): Calc!A58
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A359
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A249
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A122
- gen-m9/2138b909d8f810ea.case.json (3): Calc!A130, A184, A354
- gen-m9/31f09e6c48ca245c.case.json (2): Calc!A230, A321
- gen-m9/329b177b5d5165ad.case.json (3): Calc!A136, A148, A342
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A384
- gen-m9/3b7990bd4e3307ba.case.json (3): Calc!A252, A351, A385
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A335
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A60
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A65
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A203
- gen-m9/46ac3ef5e61583ca.case.json (2): Calc!A185, A291
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A41
- gen-m9/5be29fa316931f05.case.json (1): Calc!A157
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A368
- gen-m9/73f053591a54bc43.case.json (1): Calc!A30
- gen-m9/775d053de498b8c7.case.json (1): Calc!A297
- gen-m9/7980ffb6095a340d.case.json (2): Calc!A253, A368
- gen-m9/7f7ca726c3f68554.case.json (2): Calc!A146, A387
- gen-m9/7fd0a98183c6babf.case.json (3): Calc!A164, A260, A358
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A54
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A5
- gen-m9/941c409532603d32.case.json (2): Calc!A227, A397
- gen-m9/9735c5ab72fa5ecf.case.json (3): Calc!A102, A124, A358
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A10
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A235
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A259
- gen-m9/a4090567758520df.case.json (1): Calc!A278
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A332
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A96
- gen-m9/afc00c8b70e78baf.case.json (2): Calc!A32, A299
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A231
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A29
- gen-m9/c2684066d8df263e.case.json (1): Calc!A241
- gen-m9/c7664e494e418395.case.json (1): Calc!A177
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A163
- gen-m9/ce312817dc9c7f0b.case.json (2): Calc!A138, A384
- gen-m9/d208bbafc3bc1997.case.json (2): Calc!A243, A350
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A40
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A129, A193
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A355
- gen-m9/f82473275380882e.case.json (1): Calc!A20

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): FORECAST.ETS calls nested in the financial functions, `#N/A` on excel-graph with no argument that decides it (the enclosing financial function passes the `#N/A` on).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 40 known-failure lines in eval/baseline/gen-m9.txt, 40 visibly off.
Examples:
- gen-m9/3aa168259335be54 Calc!A200 `=COUPDAYS(0,45842,1,N(FORECAST.ETS.CONFINT(Data!C$8,Data!A3:A7,Data!B9:D10)))`: Excel #N/A, ours #N/IMPL!
- gen-m9/5ded7594e597e31e Calc!A376 `=ISPMT(FORECAST.ETS(25,Data!A3:B9,Data!C9:C12,3),5,212,298422)`: Excel #N/A, ours #N/IMPL!
- gen-m9/67a4c03afd63f988 Calc!A49 `=XNPV(FORECAST.ETS.STAT(Data!D5:E5,Data!C9:C12,7),Data!$A$6,2.5)`: Excel #N/A, ours #N/IMPL!
- gen-m9/89f0aee953a55a45 Calc!A134 `=XIRR({0,0,0},FORECAST.ETS.CONFINT(MONTH(5006),Data!D8:D9,(Data!$C$9/Data!C$3)))`: Excel #N/A, ours #N/IMPL!
- gen-m9/9a1fa5366bc50918 Calc!A36 `=NOMINAL(FORECAST.ETS.SEASONALITY(Data!C6:E8,F.INV(0.975,2.5,1),1,6),11)`: Excel #N/A, ours #N/IMPL!
- gen-m9/e8d55ec393460bb7 Calc!A24 `=AMORDEGRC(0.1,37368,39050,100,0,FORECAST.ETS.STAT(Data!E2:E4,Data!A9,1),3)`: Excel #N/A, ours #N/IMPL!
Cells:
- gen-m9/01523d247594952e.case.json (1): Calc!A201
- gen-m9/0a957119475760c5.case.json (1): Calc!A239
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A132
- gen-m9/23d82b62e9a85bdd.case.json (2): Calc!A1, A262
- gen-m9/30e20ba3302fea8b.case.json (2): Calc!A123, A349
- gen-m9/3aa168259335be54.case.json (4): Calc!A200, A219, A285, A326
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A317
- gen-m9/5422bdbf173fc233.case.json (2): Calc!A17, A235
- gen-m9/5c6a23c4f6597f1c.case.json (2): Calc!A127, A266
- gen-m9/5ded7594e597e31e.case.json (2): Calc!A98, A376
- gen-m9/6376e27be7a05871.case.json (2): Calc!A243, A345
- gen-m9/67a4c03afd63f988.case.json (2): Calc!A49, A239
- gen-m9/79d87fc527927626.case.json (1): Calc!A303
- gen-m9/7d2979fba2c95d64.case.json (2): Calc!A289, A338
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A10
- gen-m9/89f0aee953a55a45.case.json (2): Calc!A98, A134
- gen-m9/9a1fa5366bc50918.case.json (2): Calc!A36, A220
- gen-m9/a91bad4e84f49739.case.json (2): Calc!A12, A141
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A193
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A223
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A362
- gen-m9/d0854366830dcdee.case.json (1): Calc!A80
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A121, A341
- gen-m9/e8d55ec393460bb7.case.json (2): Calc!A24, A110

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): 17 more cells, a FORECAST.ETS* call with no argument error nested in an engineering or database call (BESSELI, BESSELJ, BIN2OCT, BITRSHIFT, COMPLEX, DEC2BIN, DEC2HEX, DELTA, DVAR, GESTEP, IMSEC, OCT2BIN), `#N/A` on excel-graph.
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 formulas, 17 visibly off.
Examples:
- gen-m9/060b18586fcff6e6 Calc!A232 `=BESSELI(FORECAST.ETS(12,Data!A11:B12,{1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;16;17;18;19;20;21;22;23;24},3,Data!D6,INDIRECT("Data!$C$5")),1)`: Excel #N/A, ours #N/IMPL!
- gen-m9/09e6a8028f5acfae Calc!A383 `=DEC2BIN((FORECAST.ETS(30,Data!E4,Data!$D$11)*FIND(Data!C9,Data!E3)))`: Excel #N/A, ours #N/IMPL!
- gen-m9/1068c44afcd56d53 Calc!A207 `=DEC2HEX(96018,BETADIST(FORECAST.ETS.SEASONALITY(Data!B6:C8,Data!D$1),2,SUM(Data!E9:E10),(Data!D6-Data!A5)))`: Excel #N/A, ours #N/IMPL!
- gen-m9/493bb194c8e0b577 Calc!A319 `=OCT2BIN((Data!D9^7),SUM(FORECAST.ETS.SEASONALITY(Data!B1:E3,Data!B6:B7,0,6),Data!B1:B2,SUM(Data!E9:E11,Data!E12)))`: Excel #N/A, ours #N/IMPL!
- gen-m9/4fffa5addab1f8af Calc!A90 `=BESSELJ((FORECAST.ETS.SEASONALITY({-1;-2;-3;-4;-5;-6;-7;-8;-9;-10;-11;-12;-13;-14;-15;-16;-17;-18;-19;-20;-21;-22;-23;-24},Data!A11:C11)/(Data!A5-0.3)),3)`: Excel #N/A, ours #N/IMPL!
- gen-m9/5b6cc12a94f7bf14 Calc!A77 `=DELTA(DATE(1972,8,4),(FORECAST.ETS.CONFINT(2,{1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;16;17;18;19;20;21;22;23;24;25},Data!E$11,0,1.005,Data!A1,6)*MATCH(" x ",Data!D8:D12)))`: Excel #N/A, ours #N/IMPL!
Cells:
- gen-m9/060b18586fcff6e6.case.json (1): Calc!A232
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A383
- gen-m9/1068c44afcd56d53.case.json (1): Calc!A207
- gen-m9/493bb194c8e0b577.case.json (1): Calc!A319
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A90
- gen-m9/5b6cc12a94f7bf14.case.json (2): Calc!A77, A229
- gen-m9/76b07b57099ccc8d.case.json (1): Calc!A243
- gen-m9/7a819ffbbeff8fcc.case.json (2): Calc!A174, A210
- gen-m9/895a33aa8352864a.case.json (1): Calc!A158
- gen-m9/8c018ae063ac131b.case.json (1): Calc!A21
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A21
- gen-m9/8fa8b572ebc1f001.case.json (1): Calc!A360
- gen-m9/d193e8b4ab820f52.case.json (1): Calc!A285
- gen-m9/e707d43a891ba9b5.case.json (1): Calc!A95
- gen-m9/fb73d8163ad87fc6.case.json (1): Calc!A394

More owned gen-m9 cells (gap triage 2026-10-26, `make eval` on 1c8b3d34, excel-graph): 38 FORECAST.ETS-family cells
with the family as the outer call that the list above does not name. 35 are Excel #N/A, ours #N/IMPL!; three are an
argument error that Excel returns before the family's #N/A: gen-m9/fa25422d6f18c157 Calc!A75 (Excel #REF!, ours #N/IMPL!),
A100 and A122 (Excel #VALUE!, ours #N/A). Those three are argument-order cases that T-2424 (done) did not cover; split
them into their own task if they should be fixed before the family is implemented.
Cells:
- gen-m9/22656c225ff4f22a.case.json (5): Calc!A73, A136, A196, A215, A253
- gen-m9/43a13a091907b402.case.json (4): Calc!A5, A232, A317, A387
- gen-m9/67f5e0fe790e6bec.case.json (4): Calc!A51, A289, A291, A368
- gen-m9/ec2a1d9194a84396.case.json (7): Calc!A18, A75, A136, A141, A259, A295, A310
- gen-m9/fa25422d6f18c157.case.json (18): Calc!A3, A5, A17, A29, A75, A81, A100, A122, A134, A136, A181, A212, A220, A231, A263, A278, A360, A365

T-2676 sweep (gen-m9 round 2: ACCRINT, CONVERT, GEOMEAN, VAR.P, VAR.S outer calls; gap triage 2026-10-26, excel-graph):
4 cells where a FORECAST.ETS-family call with no argument error of its own is nested in the outer call, which passes
the web's `#N/A` on (ours `#N/IMPL!`). The other arguments were evaluated with the engine and none decides the result
before it.
- gen-m9/290180acf017709c Calc!A98 `=CONVERT((FORECAST.ETS.SEASONALITY(Data!D9:D11,Data!B$12)+((-1)/Data!C1)),INDEX(Data!C2:D5,3,WORKDAY(38128,7)),"F")`:
  Excel #N/A, ours #N/IMPL! (CONVERT reports the number's error first; the from_unit is #REF!)
- gen-m9/932b33b1ef8b7608 Calc!A336 `=VAR.S(LOG10(FORECAST.ETS.STAT((-2.5),Data!E5,5,4)),{100000000.1,100000000.2,100000000.3},MIN(Data!E10:E11))`:
  Excel #N/A, ours #N/IMPL!
- gen-m9/f81bd3b6dc373e5c Calc!A64 `=GEOMEAN((Data!$C10-FORECAST.ETS.STAT(Data!E1:E12,{1;2;...;24},Data!D2,2,0,5)),{3,3,3,3},TEXTBEFORE(..))`:
  Excel #N/A, ours #N/IMPL!
- gen-m9/f81bd3b6dc373e5c Calc!A331 `=CONVERT(WORKDAY(54399,-27,FORECAST.ETS.CONFINT(Data!$E2,7,Data!E4)),"dyn","yd")`:
  Excel #N/A, ours #N/IMPL!
Cells:
- gen-m9/290180acf017709c.case.json (1): Calc!A98
- gen-m9/932b33b1ef8b7608.case.json (1): Calc!A336
- gen-m9/f81bd3b6dc373e5c.case.json (2): Calc!A64, A331
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/22656c225ff4f22a.case.json: Calc!A194, A36, A124, A156, A35, A155, A183, A231, A367, A105, A130, A139, A169, A273, A280, A308, A329, A334
  - gen-m9/146b46e74f2dac3b.case.json: Calc!A174
  - gen-m9/165db58d1540f014.case.json: Calc!A19, A145
  - gen-m9/4250f880ab640c49.case.json: Calc!A340
  - gen-m9/43a13a091907b402.case.json: Calc!A4, A61, A67, A91, A107, A165, A392
  - gen-m9/44a8841352dc48e9.case.json: Calc!A251
  - gen-m9/60b0b6e69ad35a45.case.json: Calc!A113
  - gen-m9/67f5e0fe790e6bec.case.json: Calc!A14, A37, A198, A272, A298, A336
  - gen-m9/899f9555e67da2d6.case.json: Calc!A142
  - gen-m9/941c409532603d32.case.json: Calc!A324
  - gen-m9/c2684066d8df263e.case.json: Calc!A318
  - gen-m9/caa1eed48b1883fe.case.json: Calc!A61
  - gen-m9/ec2a1d9194a84396.case.json: Calc!A67, A218, A316
  - gen-m9/fd9fa22d212080d6.case.json: Calc!A58

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 17 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): the 40 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 940 known-failure lines (877 from T-2255, 63 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2676): the 4 known-failure lines listed under "T-2676 sweep" in Context disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Desktop values for these cases exist (T-1553) and the cells are scored against them; the family's algorithm is
      specified (docs/specs/) before it is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The argument step (T-2424).
- Any implementation that returns the web `#N/A` without desktop evidence (T-1293's Outcome).

## Blocked because
Needs desktop Excel values for these cases: T-1553 (the `excel-desktop` backend, a Windows machine with Excel; decision
0039). Proof it is available: `cd tools && uv run rustsheet-oracle status` lists an `excel-desktop` backend as ready.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Milestone (decision 0114)
Moved from M9 to M14: the values need desktop Excel (excel-graph leaves them `#N/A`), so under decision 0039 rule 3 this filed task does not block the Phase 1 exit. Its argument-error cells stay with T-2424 (`campaign: semantics`).

## Cells added by the Phase 1 triage (T-0090, 2026-10-27)
At the 2026-10-27 eval 302 visibly-off FORECAST.ETS, FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT and FORECAST.ETS.CONFINT cells were owned by no open task, so these four functions could not meet the "every remaining failure has a filed task" rule. This task owns them too (one line per case, cells listed):
- gen-m9/43a13a091907b402: A10, A11, A18, A24, A27, A34, A35, A37, A49, A50, A52, A54, A62, A63, A64, A66, A70, A71, A74, A78, A79, A82, A86, A87, A90, A92, A94, A95, A99, A111, A115, A120, A121, A127, A130, A134, A138, A142, A146, A150, A151, A158, A159, A171, A182, A183, A187, A188, A194, A202, A218, A226, A228, A243, A251, A254, A255, A258, A260, A261, A262, A270, A271, A275, A278, A282, A294, A298, A302, A306, A311, A323, A325, A326, A338, A339, A348, A350, A351, A353, A354, A359, A362, A363, A366, A375, A381, A387, A395, A396, A397
- gen-m9/67f5e0fe790e6bec: A8, A11, A20, A22, A23, A27, A34, A41, A51, A52, A55, A56, A59, A62, A67, A71, A78, A83, A86, A90, A94, A95, A98, A99, A102, A107, A108, A109, A114, A134, A138, A146, A156, A164, A166, A170, A175, A179, A182, A189, A199, A200, A219, A220, A225, A227, A230, A235, A239, A242, A243, A246, A250, A254, A257, A263, A266, A271, A278, A286, A287, A289, A290, A291, A297, A299, A300, A302, A303, A314, A343, A354, A355, A358, A359, A361, A362, A363, A384, A386, A390, A391, A395
- gen-m9/ec2a1d9194a84396: A2, A11, A12, A14, A21, A22, A24, A32, A34, A40, A42, A44, A45, A46, A49, A52, A58, A61, A68, A74, A76, A78, A79, A88, A90, A92, A98, A100, A104, A112, A113, A114, A122, A123, A124, A129, A132, A134, A152, A158, A160, A166, A171, A176, A180, A182, A186, A198, A210, A212, A214, A224, A228, A231, A244, A252, A255, A256, A258, A273, A274, A278, A280, A282, A284, A286, A288, A294, A300, A306, A308, A318, A320, A334, A335, A339, A340, A341, A342, A351, A373, A380, A384, A385, A386, A390, A398
- probes/forecast-ets-longer-series: Sheet1!K1, Sheet1!K2, Sheet1!K3, Sheet1!K4, Sheet1!K5, Sheet1!K6, Sheet1!K7, Sheet1!K8, Sheet1!K9, Sheet1!K10, Sheet1!K11, Sheet1!K12, Sheet1!K13, Sheet1!K14, Sheet1!K15, Sheet1!K16, Sheet1!K17, Sheet1!K18, Sheet1!K19, Sheet1!K20, Sheet1!K21, Sheet1!K22, Sheet1!K23, Sheet1!K24, Sheet1!K25, Sheet1!K26, Sheet1!K27, Sheet1!K28, Sheet1!K29, Sheet1!K30, Sheet1!K31, Sheet1!K32, Sheet1!K33, Sheet1!K34, Sheet1!K35, Sheet1!K36, Sheet1!K37, Sheet1!K39, Sheet1!K40, Sheet1!K41, Sheet1!K42

T-0085 makes the eval's Phase 1 line treat the cells of this desktop-evidence task as filed rather than as misses, as decision 0114 says.

## Cells added by decision 0188 (2026-10-31)
The FORECAST.ETS cells of the statistical argument-modes probe below, `#N/A` on excel-graph like every valid call of the family
(statistics.md "FORECAST.ETS family: excel-graph never computes it"), are this task's. T-0969 had owned the whole case
through a line naming it without a cell; it now names only its own cells, so these are owned by this M14 task alone
(decision 0143; R7 of decision 0188 for the M9 probe row):
- probes/stat-argument-modes.case.json (10): Sheet1!H266, H267, H270, H271, H274, H275, H276, H277, H278, H281
