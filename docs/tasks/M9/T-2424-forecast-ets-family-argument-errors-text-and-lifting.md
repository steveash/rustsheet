---
id: T-2424
title: "FORECAST.ETS family: argument errors, text and lifting decide the result before #N/IMPL! (gen-m9, 1196 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2028]
campaign: semantics
---

## Goal
The four FORECAST.ETS functions return what Excel for the web returns before it gives up on the forecast: the first
error argument's error in argument order, `#VALUE!` for a text scalar argument in its turn, and a lifted array over an
array or multi-cell range in a scalar argument; only a call with none of these keeps `#N/IMPL!` (T-2425, desktop values).

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "FORECAST.ETS family on excel-graph: argument errors, text and lifting
decide before the #N/A (T-2255)" records the rule from these cells: of the 1,173 top-level cells, the first error argument
decides 967 (target_date or values 479, timeline 219, then later arguments), a text scalar 36 (`FORECAST.ETS(Data!C$4,
BYROW(..)<#CALC!>,..)` with C4 = "ABC" is `#VALUE!`), a lifted scalar argument 54 (`#SPILL!` in the generated layout), and
116 have an error element inside values/timeline or an argument whose own function differs from ours (fix those inner
functions first if one of them is the cause: T-2093, T-2426, T-2427, T-2428, T-2436 own the same inner shapes elsewhere);
23 cells nest a FORECAST.ETS* call whose own argument is an error. This is not the validation-only implementation T-1293
ruled out: a web `#N/A` with no error argument stays `#N/IMPL!` (T-2425), and every rule here is what desktop Excel must
do too before it computes (an argument error cannot be overruled by a later forecast).
Builds on T-2028 (lifting of target_date/statistic_type and the text target_date `#VALUE!` through `lift_call`, a kernel
that returns `#N/IMPL!` after the number coercion). Code: `fn_forecast_ets*` in crates/rustsheet-engine/src/functions/
statistical/forecast.rs; the lift kinds in lift/kinds/statistical.rs (regenerate eval/catalog/lift-kinds.json). values and
timeline are array arguments: an error value given directly is that error; an error element inside them is outside this
task's evidence where the web result is `#N/A` (count those cells in the Outcome).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1196 known-failure lines in eval/baseline/gen-m9.txt, 1196 formulas.
Examples:
- gen-m9/146b46e74f2dac3b Calc!A174 `=LOGINV(1E-16,FORECAST.ETS.STAT(Data!A8:D11,Data!E10:E10,7),2)`: Excel #N/A, ours #N/IMPL!
- gen-m9/165db58d1540f014 Calc!A103 `=LOGNORMDIST(COT((-PI())),FORECAST.ETS(42,INDEX(Data!A7:D11,Data!B$6,Data!D$2),(1000000000000000.0-Data!E$1)),100)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/22656c225ff4f22a Calc!A111 `=FORECAST.ETS.STAT(Data!A6:C8,Data!B$8,8,(-0.0001),0,HARMEAN(Data!B5:B9))`: Excel #NUM!, ours #N/IMPL!
- gen-m9/22656c225ff4f22a Calc!A118 `=FORECAST.ETS.CONFINT(UNIQUE((Data!$D$10*Data!E6),(Data!E5>="abc"),(Data!E10<"ü")),Data!D6:D12,Data!B8:D12,HSTACK((Data!D5/Data!$A$7),IMREAL(Data!A1),(Data!D9>=Data!$C1)))`: Excel #DIV/0!, ours #N/IMPL!
- gen-m9/22656c225ff4f22a Calc!A137 `=FORECAST.ETS(TRANSPOSE(Data!A2:D4),GETPIVOTDATA("count of sales",Pivot!$H$3,"Rep"),Data!C3:D10,1,1)`: Excel #SPILL!, ours #N/IMPL!
- gen-m9/22656c225ff4f22a Calc!A157 `=FORECAST.ETS(((1.005-(-1))-INDEX(Data!D10:D12,6)),Data!A9:C12,(-0.2/Data!E5),0)`: Excel #REF!, ours #N/IMPL!
Cells:
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A174
- gen-m9/165db58d1540f014.case.json (4): Calc!A19, A103, A145, A244
- gen-m9/1782efcecd5d4b1b.case.json (1): Calc!A27
- gen-m9/22656c225ff4f22a.case.json (239): Calc!A1, A2, A3, A5, A6, A8, A9, A10, A11, A12, A13, A14, A17, A18, A22, A24, A25, A28, A29, A30, A31, A32, A35, A36, A37, A38, A40, A42, A44, A46, A54, A55, A57, A58, A59, A60, A61, A65, A67, A69, A70, A72, A73, A74, A76, A78, A82, A83, A85, A86, A88, A89, A90, A91, A93, A94, A95, A98, A100, A102, A104, A105, A110, A111, A113, A114, A116, A118, A119, A121, A122, A123, A124, A126, A130, A131, A132, A133, A134, A136, A137, A138, A139, A141, A142, A146, A147, A148, A150, A154, A155, A156, A157, A158, A161, A162, A165, A166, A167, A169, A170, A171, A172, A173, A174, A175, A177, A180, A181, A183, A184, A185, A186, A189, A190, A192, A193, A194, A195, A196, A197, A198, A200, A201, A202, A203, A204, A205, A207, A209, A213, A214, A215, A217, A218, A220, A221, A222, A223, A224, A231, A232, A235, A238, A240, A243, A245, A246, A247, A249, A250, A253, A256, A257, A258, A259, A260, A261, A263, A264, A265, A266, A268, A269, A273, A274, A276, A277, A280, A281, A285, A286, A287, A289, A290, A291, A293, A294, A295, A297, A299, A300, A301, A302, A303, A305, A306, A308, A310, A311, A312, A313, A318, A321, A322, A325, A327, A328, A329, A330, A331, A333, A334, A337, A338, A339, A342, A344, A346, A347, A350, A351, A352, A353, A354, A355, A357, A358, A362, A363, A367, A370, A371, A372, A373, A375, A376, A377, A378, A382, A384, A385, A389, A390, A391, A392, A394, A396, A398
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A369
- gen-m9/3462856259a37891.case.json (1): Calc!A28
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A10, A182
- gen-m9/4250f880ab640c49.case.json (1): Calc!A340
- gen-m9/43a13a091907b402.case.json (220): Calc!A1, A2, A3, A4, A5, A6, A8, A9, A12, A14, A16, A18, A19, A20, A21, A25, A28, A31, A32, A37, A40, A41, A43, A44, A46, A47, A48, A53, A56, A57, A60, A61, A65, A67, A68, A69, A73, A76, A77, A80, A81, A83, A84, A85, A89, A91, A93, A97, A98, A100, A101, A103, A104, A105, A107, A109, A112, A114, A116, A117, A118, A119, A122, A123, A124, A125, A127, A128, A129, A131, A132, A136, A137, A140, A141, A143, A144, A145, A148, A153, A156, A157, A160, A161, A162, A163, A165, A168, A169, A170, A172, A176, A178, A180, A181, A184, A185, A186, A189, A190, A191, A193, A195, A196, A197, A198, A199, A200, A201, A203, A204, A205, A206, A207, A209, A210, A211, A215, A216, A217, A220, A221, A222, A223, A224, A229, A230, A231, A232, A233, A234, A236, A237, A238, A239, A240, A241, A245, A246, A247, A248, A250, A252, A253, A256, A257, A263, A267, A268, A272, A273, A274, A276, A277, A279, A283, A284, A285, A288, A289, A291, A292, A293, A296, A297, A301, A302, A308, A310, A312, A313, A314, A315, A316, A317, A318, A319, A321, A322, A324, A328, A331, A332, A333, A334, A336, A337, A340, A341, A343, A345, A346, A347, A349, A352, A355, A356, A359, A360, A361, A365, A368, A369, A371, A372, A376, A377, A378, A379, A380, A385, A387, A388, A389, A390, A392, A394, A398, A399, A400
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A292
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A4
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A113
- gen-m9/67f5e0fe790e6bec.case.json (243): Calc!A1, A2, A3, A5, A6, A7, A9, A12, A13, A14, A16, A17, A19, A21, A25, A26, A29, A30, A31, A32, A33, A36, A37, A38, A39, A40, A42, A43, A45, A46, A47, A48, A49, A50, A51, A53, A54, A57, A60, A61, A63, A64, A65, A68, A69, A70, A73, A74, A75, A76, A77, A79, A80, A81, A84, A85, A88, A89, A91, A96, A97, A100, A101, A103, A104, A105, A112, A113, A115, A116, A117, A118, A120, A122, A123, A124, A125, A126, A127, A128, A129, A130, A132, A133, A135, A136, A137, A141, A142, A143, A145, A149, A150, A152, A153, A154, A157, A158, A160, A162, A163, A165, A169, A172, A173, A174, A176, A177, A178, A180, A183, A185, A187, A188, A192, A193, A194, A195, A196, A198, A201, A204, A205, A206, A209, A210, A211, A212, A215, A216, A217, A221, A223, A224, A226, A227, A228, A229, A231, A232, A233, A234, A236, A237, A238, A241, A244, A245, A247, A248, A249, A250, A251, A253, A255, A256, A258, A259, A260, A261, A264, A265, A267, A268, A269, A270, A272, A273, A274, A277, A279, A280, A282, A283, A284, A285, A288, A289, A291, A292, A293, A296, A298, A301, A304, A305, A306, A307, A308, A309, A313, A316, A317, A318, A319, A320, A322, A323, A324, A325, A327, A328, A330, A332, A334, A335, A336, A337, A338, A341, A342, A344, A345, A346, A347, A348, A349, A353, A356, A357, A359, A360, A366, A368, A369, A372, A374, A375, A376, A377, A378, A379, A380, A382, A383, A385, A387, A389, A392, A393, A397, A399, A400
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A142
- gen-m9/a54c738c697617c3.case.json (1): Calc!A75
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A296
- gen-m9/b4173fe25109a238.case.json (1): Calc!A101
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A321
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A161
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A61
- gen-m9/ec2a1d9194a84396.case.json (215): Calc!A1, A3, A5, A6, A7, A9, A10, A13, A18, A19, A20, A23, A25, A27, A28, A29, A30, A31, A33, A37, A38, A39, A41, A43, A48, A51, A56, A57, A59, A60, A62, A63, A64, A65, A66, A67, A70, A71, A73, A74, A75, A77, A80, A83, A84, A85, A86, A87, A89, A91, A93, A97, A99, A101, A106, A107, A108, A110, A119, A120, A131, A133, A136, A137, A138, A139, A140, A141, A144, A146, A147, A148, A149, A151, A153, A154, A155, A156, A157, A159, A161, A162, A163, A164, A165, A167, A173, A174, A177, A178, A179, A183, A184, A185, A189, A190, A191, A193, A195, A196, A199, A201, A203, A204, A205, A206, A208, A209, A211, A215, A217, A218, A222, A225, A227, A229, A230, A232, A233, A234, A236, A237, A239, A240, A241, A243, A245, A246, A247, A248, A253, A257, A259, A260, A261, A262, A263, A267, A268, A269, A270, A272, A275, A277, A279, A281, A283, A285, A287, A289, A291, A292, A295, A297, A298, A299, A301, A302, A303, A304, A305, A307, A309, A310, A311, A312, A315, A316, A317, A319, A321, A322, A323, A324, A325, A326, A327, A329, A330, A332, A333, A336, A337, A343, A344, A345, A347, A348, A349, A350, A352, A353, A356, A360, A361, A362, A363, A368, A369, A370, A371, A377, A379, A381, A382, A383, A387, A388, A389, A391, A392, A393, A395, A397, A399
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A340
- gen-m9/fa25422d6f18c157.case.json (254): Calc!A1, A2, A3, A4, A5, A7, A9, A13, A15, A17, A18, A19, A23, A24, A25, A26, A27, A28, A29, A30, A31, A33, A34, A36, A39, A40, A41, A43, A46, A47, A48, A49, A50, A51, A52, A54, A55, A57, A58, A59, A61, A63, A64, A66, A69, A72, A73, A75, A76, A77, A78, A81, A82, A83, A84, A85, A86, A87, A92, A93, A94, A95, A100, A101, A103, A105, A107, A108, A110, A111, A112, A113, A115, A117, A118, A120, A121, A122, A123, A124, A125, A126, A127, A129, A132, A133, A134, A135, A136, A139, A140, A141, A143, A145, A147, A150, A152, A153, A154, A155, A157, A158, A159, A160, A161, A162, A164, A167, A168, A169, A170, A171, A174, A175, A178, A179, A180, A181, A182, A184, A190, A191, A194, A195, A196, A197, A198, A201, A203, A204, A205, A206, A208, A209, A210, A211, A212, A214, A217, A219, A220, A221, A222, A223, A225, A226, A229, A231, A232, A233, A234, A236, A237, A238, A239, A240, A241, A242, A243, A245, A249, A250, A253, A259, A260, A261, A262, A263, A266, A270, A271, A274, A278, A279, A281, A283, A286, A287, A289, A290, A291, A292, A293, A294, A295, A296, A300, A302, A303, A305, A307, A309, A310, A311, A312, A314, A315, A317, A318, A319, A321, A322, A323, A324, A326, A327, A329, A330, A331, A333, A334, A335, A339, A343, A344, A345, A346, A347, A348, A350, A351, A352, A354, A355, A356, A357, A358, A359, A360, A361, A362, A363, A364, A365, A366, A367, A368, A369, A373, A374, A375, A379, A383, A385, A386, A387, A388, A389, A391, A392, A393, A396, A397, A398
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A58, A222
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A213

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): FORECAST.ETS* calls nested in another function whose own argument error, text or lift decides the outer result (`STANDARDIZE(FORECAST.ETS.SEASONALITY(..,0.5),..)` is `#VALUE!`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 19 known-failure lines in eval/baseline/gen-m9.txt, 19 visibly off.
Examples:
- gen-m9/139684e68fb2fc3b Calc!A227 `=PEARSON(FORECAST.ETS.SEASONALITY(MATCH("#N/A",Data!B5:E6,-1),Data!$B6,1),Data!C7:D9)`: Excel #N/A, ours #N/IMPL!
- gen-m9/5389dd4d5b5739d2 Calc!A210 `=MINA(FORECAST.ETS.SEASONALITY(DATEDIF(23881,3410,"D"),Data!A11:B12),Data!A1:E1)`: Excel #NUM!, ours #N/IMPL!
- gen-m9/607e85cfee83a689 Calc!A201 `=SMALL(FORECAST.ETS.STAT(Data!C1:E1,Data!A11:D11,(Data!E$11*Data!D6)),4)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/eedd629f3d69fdda Calc!A374 `=NORMSDIST(FORECAST.ETS(Data!B2,(1-Data!$A1),(Data!E9-Data!D1),1,0))`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/f82473275380882e Calc!A393 `=PROB(Data!C3:C7,OFFSET(Data!B5:D11,2,2),FORECAST.ETS.STAT(2.5,Data!A4:A11,4,4,Data!$E$3,6))`: Excel #VALUE!, ours #N/IMPL!
Cells:
- gen-m9/09b47629f54b9817.case.json (1): Calc!A194
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A227
- gen-m9/17c110a53924beb8.case.json (1): Calc!A200
- gen-m9/1db18890916ee643.case.json (1): Calc!A312
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A400
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A251 `TTEST(..,FORECAST.ETS.SEASONALITY({-1;..;-24},..,0),1,..)`: Excel #N/A, ours #N/IMPL! (moved from T-2469)
- gen-m9/5389dd4d5b5739d2.case.json (2): Calc!A89, A210
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A87
- gen-m9/607e85cfee83a689.case.json (2): Calc!A201, A355
- gen-m9/775d053de498b8c7.case.json (1): Calc!A51
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A49
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A248
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A257
- gen-m9/a4090567758520df.case.json (1): Calc!A5
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A81
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A145
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A374
- gen-m9/f82473275380882e.case.json (1): Calc!A393

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): FORECAST.ETS calls nested in the financial functions whose Excel error comes from an ETS argument (`#VALUE!`, `#NUM!`, `#DIV/0!`), where ours is `#N/IMPL!`.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 15 known-failure lines in eval/baseline/gen-m9.txt, 15 visibly off.
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A109 `=MDURATION(37604,45422,FORECAST.ETS(2958465,MONTH(Data!B1),Data!B2,2,(Data!B6/Data!E4),5),1E+300,2,2)`: Excel #NUM!, ours #N/IMPL!
- gen-m9/3aa168259335be54 Calc!A82 `=YIELD(36857,LET(v,JIS("1"),y,MOD(v,v),acc,-Data!A9,(0.1-v)),1E+10,110,108,2,Data!B$11)`: Excel #NUM!, ours #NAME?
- gen-m9/67a4c03afd63f988 Calc!A150 `=IRR(FORECAST.ETS.SEASONALITY(SUMIFS(Data!B4:E10,Data!B8:E12,"=a",Data!D4:E7,"<5",Data!E4:E11,"<5"),Data!A11:C11),1)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/81bcdc0bec0c8e6b Calc!A371 `=COUPNCD(39641,44319,FORECAST.ETS(24,(Data!C2^Data!B8),Data!D8:E12,1,0),1)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/9d9ceba257f8001b Calc!A66 `=DURATION(39458,46437,FORECAST.ETS.CONFINT((Data!$E7-Data!A2),Data!C3:D12,Data!A2:D6,Data!E3),1E-9,3)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/e168be7f6d3f84c2 Calc!A170 `=MDURATION(38966,46752,7,FORECAST.ETS.CONFINT((Data!A9+Data!$B$6),Data!B6:D12,(-1),EVEN(2.675)),4,2)`: Excel #VALUE!, ours #N/IMPL!
Cells:
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A109
- gen-m9/23d82b62e9a85bdd.case.json (1): Calc!A334
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A194
- gen-m9/3aa168259335be54.case.json (1): Calc!A82
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A299
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A150
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A371
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A247
- gen-m9/9d9ceba257f8001b.case.json (2): Calc!A66, A71
- gen-m9/a91bad4e84f49739.case.json (3): Calc!A190, A326, A400
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A112
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A170

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): 4 more cells nested in engineering calls where an argument error decides: `FORECAST.ETS.STAT(..,MONTH(1E+15)<#NUM!>,2)` `#NUM!`, `FORECAST.ETS.CONFINT(<#VALUE!>,..)` `#VALUE!`, `FORECAST.ETS.CONFINT(25,..,ACOS("Hello World")<#VALUE!>,..)` `#VALUE!`, `FORECAST.ETS.STAT(..,<#VALUE!>,..)` `#VALUE!`.
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas, 4 visibly off.
Examples:
- gen-m9/4e180f54ff140abc Calc!A103 `=COMPLEX(FORECAST.ETS.STAT(Data!A4:D5,MONTH(Data!C4),2),SUMPRODUCT(Data!C9:E11))`: Excel #NUM!, ours #N/IMPL!
- gen-m9/5bdef392d16a5efb Calc!A18 `=DEC2HEX(71256,FORECAST.ETS.CONFINT((Data!B8*Data!B1),Data!B6:E12,Data!C8:E11,MATCH(Data!$E12,Data!C3:C12,1),0,0,2))`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/bf979a17ab5aafcb Calc!A266 `=DELTA(FORECAST.ETS.CONFINT(25,{1;4;9;16;25;36;49;64;81;100;121;144;169;196;225;256;289;324;361;400;441;484;529;576},ACOS(Data!C11),IFNA("1.5",Data!A7)),0)`: Excel #VALUE!, ours #N/IMPL!
- gen-m9/f8bc18a0e45a0e1a Calc!A123 `=BITXOR(36,FORECAST.ETS.STAT(Data!B2:C4,(Data!B$8*0.5),1,4,0,(Data!C12-0.5)))`: Excel #VALUE!, ours #N/IMPL!
Cells:
- gen-m9/4e180f54ff140abc.case.json (1): Calc!A103
- gen-m9/5bdef392d16a5efb.case.json (1): Calc!A18
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A266
- gen-m9/f8bc18a0e45a0e1a.case.json (1): Calc!A123

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): the 15 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 1215 known-failure lines (1196 from T-2255, 19 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: an error in each argument position (first one wins), a text target_date/seasonality/statistic_type
      before a later error, and a 2-row target_date that spills; a call with no error or text stays `#N/IMPL!`.
- [x] statistics.md's T-2255 entry gets an "Ours:" line naming this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any forecast value or the web `#N/A` of a valid call (T-2425, after T-1553).
- The inner functions' own differences (their tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
`statistical/forecast.rs` `forecast_ets_lifted` is the argument step for all four functions. Each element reads the
arguments in order, and the first that fails decides:
- a scalar argument's error;
- non-numeric text (`#VALUE!`);
- an error given directly as `values` or `timeline`, which are evaluated once.

Only a call with none of these stays `#N/IMPL!`. Every scalar argument lifts. FORECAST.ETS.SEASONALITY gets the lift
entry `ETS_SEASONALITY`, so its data_completion and aggregation lift too, and `eval/catalog/lift-kinds.json` is
regenerated. Unit test `forecast_ets_family_first_failing_argument_decides`; statistics.md's T-2255 entry has an
"Ours:" line.

Eval (main 2c34755d, 2026-10-25): gen-m9 5,223 -> 4,567 known failures (656 fixed), 0 regressions and no new lines
in any baseline. In-scope cells visibly off: 4,872 (lock 6,039).

Of the 1,234 owned gen-m9 cells, 97 still differ, and none is this step's to fix, so the visible-parity criteria
stay unticked:
- 84 are a web `#N/A` against ours `#N/IMPL!`. They are calls with no error argument (T-2425's values, after
  T-1553), or with an error element inside `values` or `timeline`, which the web evidence does not settle (Context).
- 13 differ in an inner function that their own tasks own. Examples: GETPIVOTDATA `#REF!` in 43a13a091907b402 A4
  and 899f9555e67da2d6 A142, KURT in 43a13a091907b402 A256, XIRR in fa25422d6f18c157 A122, MODE.MULT in A100, NPV
  at rate -1 in ec2a1d9194a84396 A38, SYD in A65, and PROB in 67f5e0fe790e6bec A50, 1db18890916ee643 A312 and
  5389dd4d5b5739d2 A89.

The 13 inner-function cells are owned by those functions' own tasks (statistics.md, the T-2255 FORECAST.ETS section
lists them); none needs a FORECAST.ETS change.
