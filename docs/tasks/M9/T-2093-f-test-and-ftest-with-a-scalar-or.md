---
id: T-2093
title: "F.TEST and FTEST with a scalar or text argument are #DIV/0!, not #VALUE!"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
F.TEST and FTEST return `#DIV/0!` when either argument is a plain number or a text, as Excel does,
instead of `#VALUE!`.

## Context
Gap triage 2026-10-20. Excel treats a scalar argument as a sample with fewer than two points.
docs/excel-semantics/statistics.md, "Thin distribution functions ... (T-1049)", "Test functions",
already says this: "F.TEST: `#DIV/0!` for fewer than two points on either side, ... scalars, an
empty range or text-only input (`F.TEST("a",B1:B5)`)". No task implemented it.
Failing cells (excel-graph; probes-numeric is not gated and has no baseline file, so read them with
`make eval-show CASE=probes-numeric/<case>.case.json`). Each is Excel `#DIV/0!` and ours `#VALUE!`:
- probes-numeric/t1049-ftest Calc!A29 `F.TEST(B1:B5,3)`, A30 `F.TEST(3,B1:B5)`, A31 `F.TEST(3,4)`,
  A32 `F.TEST("a",B1:B5)`.
- probes-numeric/m9-legacy-ftest Calc!A32 `FTEST(1,B1:B5)`.
Cells that already pass and must keep passing: t1049-ftest A15 `F.TEST(B1:B1,C1:C5)` and A16 (a
one-cell range gives `#DIV/0!`), and A22 `F.TEST(B1:B5,Z1:Z5)` (an empty range).
Code: `fn_f_test` in crates/rustsheet-engine/src/functions/statistical/fisher.rs. The `_ =>` arm of
each argument's match returns `#VALUE!` for any non-range, non-array value. A number or a text
should instead go on to the existing `n1 < 2 || n2 < 2` check, which gives `#DIV/0!`. The simplest
way is to read it as a sample with no numbers. FTEST (`Function::FTestCompat`) shares `fn_f_test`.
Leave a boolean or an error value as it is now: no probe shows what Excel does with
`F.TEST(TRUE,B1:B5)` or `F.TEST(NA(),B1:B5)`. (Superseded by the gen-m9 evidence below, T-2255: a boolean is one value, an
error argument is its error.)

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 0 numeric past half a unit in the 15th digit and 1 with a wrong type, error or text, blamed on FTEST 1. Every one is listed above.

gen-m9 sweep (gap triage 2026-10-23, T-2255). F.TEST and FTEST, in 409 cells: a plain number, a one-cell reference holding a number, text
or a boolean, or a text/boolean value is the one-point `#DIV/0!` (295 cells: 249 numbers or numeric cells, 28 booleans, 18
texts; ours `#VALUE!`); an error argument is that error, the first argument's first (97 cells: `#NUM!` 41, `#N/A` 30,
`#DIV/0!` 15, `#REF!` 9, `#CALC!` 2; this was out of scope above as unprobed, gen-m9 is the evidence); an error element
inside an array argument is that error, before the one-point check (`FTEST({1,#N/A,3},AVERAGE(0.2))` `#N/A`; 17 cells with
`#N/A`). 21 of the cells nest the F.TEST in another call. statistics.md "F.TEST: an error argument is that error;
overflowing squares are #NUM!, KURT's #DIV/0! (T-2255)" and "An error element of an array argument is that error (T-2255)".
An empty one-cell reference is T-2426's (`#VALUE!`), the overflow T-2432's, three unexplained `#NUM!` cells T-2436's.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 409 known-failure lines in eval/baseline/gen-m9.txt, 409 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A105 `=FTEST(VLOOKUP(Data!D1,Data!A3:C12,3)%,Data!B3:B7)`: Excel #N/A, ours #VALUE!
- gen-m9/05d44f6dd6cfc567 Calc!A112 `=F.TEST({-1,0,1},FACT(1))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/05d44f6dd6cfc567 Calc!A234 `=F.TEST(VLOOKUP(Data!E10,Data!D4:D11,2,(0.5<Data!B$12)),Data!A10:D11)`: Excel #REF!, ours #VALUE!
- gen-m9/0a614cfe974b59c3 Calc!A143 `=F.TEST(SMALL(Data!A4:D11,CEILING(Data!C6,Data!$C11)),Data!A6:B10)`: Excel #NUM!, ours #VALUE!
- gen-m9/27601819397dd56e Calc!A360 `=FTEST(ROW(),-DROP((-1),1,42))`: Excel #CALC!, ours #VALUE!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (7): Calc!A44, A51, A105, A112, A166, A173, A234
- gen-m9/06b4e8f5de409d5e.case.json (5): Calc!A73, A200, A256, A378, A383
- gen-m9/0a614cfe974b59c3.case.json (7): Calc!A21, A82, A143, A145, A204, A206, A387
- gen-m9/0a9226016e8f5f33.case.json (8): Calc!A9, A197, A239, A243, A285, A335, A377, A381
- gen-m9/146b46e74f2dac3b.case.json (6): Calc!A92, A170, A214, A292, A353, A397
- gen-m9/165db58d1540f014.case.json (8): Calc!A49, A61, A107, A153, A199, A323, A325, A337
- gen-m9/27601819397dd56e.case.json (3): Calc!A22, A344, A360
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A290
- gen-m9/285d87cea1aaa73c.case.json (5): Calc!A41, A163, A193, A224, A346
- gen-m9/2a18b2f5a4dc43ac.case.json (6): Calc!A62, A69, A245, A313, A367, A374
- gen-m9/320d51765af9b6b7.case.json (9): Calc!A39, A90, A100, A131, A238, A269, A284, A315, A330
- gen-m9/35e8168aae2b00f8.case.json (4): Calc!A53, A175, A236, A253
- gen-m9/3a25a03eb94fcfbd.case.json (5): Calc!A52, A69, A212, A244, A374
- gen-m9/3e98b26cf6713e77.case.json (6): Calc!A34, A69, A191, A252, A278, A313
- gen-m9/40425341c5034eed.case.json (7): Calc!A10, A56, A125, A148, A217, A263, A309
- gen-m9/4250f880ab640c49.case.json (8): Calc!A20, A23, A142, A145, A203, A264, A267, A328
- gen-m9/4ab2ba98d26c8a5e.case.json (7): Calc!A17, A63, A117, A243, A247, A293, A385
- gen-m9/4bad303d32cc3f46.case.json (5): Calc!A22, A156, A278, A327, A339
- gen-m9/4e55500bb8ad3cd8.case.json (8): Calc!A92, A105, A138, A151, A184, A276, A368, A381
- gen-m9/530c636749a85209.case.json (5): Calc!A42, A126, A264, A272, A318
- gen-m9/589f42e67f4b4b8d.case.json (9): Calc!A22, A25, A68, A71, A160, A163, A206, A298, A344
- gen-m9/58e3253bbd3a7a82.case.json (9): Calc!A3, A49, A55, A101, A193, A279, A325, A331, A377
- gen-m9/60b0b6e69ad35a45.case.json (4): Calc!A172, A178, A239, A294
- gen-m9/623663cacea53bbc.case.json (7): Calc!A34, A66, A127, A156, A188, A217, A371
- gen-m9/68d8ffb69ff739d8.case.json (5): Calc!A82, A204, A265, A270, A331
- gen-m9/7674c89c824aca43.case.json (9): Calc!A9, A55, A131, A147, A239, A269, A331, A361, A377
- gen-m9/7bd219ec0c0c5e22.case.json (10): Calc!A37, A46, A47, A83, A129, A139, A175, A231, A267, A323
- gen-m9/7c20351562a2d71d.case.json (7): Calc!A103, A179, A225, A258, A304, A350, A396
- gen-m9/85f48d5cdfb24555.case.json (8): Calc!A2, A32, A124, A185, A215, A221, A246, A368
- gen-m9/899f9555e67da2d6.case.json (5): Calc!A27, A45, A269, A321, A367
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A54, A365
- gen-m9/8dd2312c96effb1b.case.json (6): Calc!A97, A116, A177, A219, A238, A299
- gen-m9/8f2ac25945af87da.case.json (5): Calc!A9, A70, A192, A314, A363
- gen-m9/8fec7b207f4dd753.case.json (8): Calc!A15, A35, A41, A76, A259, A320, A340, A381
- gen-m9/9cca17438a7e6f8a.case.json (6): Calc!A29, A43, A165, A186, A266, A348
- gen-m9/a013a45a8fbcab4f.case.json (5): Calc!A14, A159, A197, A261, A380
- gen-m9/a54c738c697617c3.case.json (7): Calc!A34, A156, A217, A223, A284, A339, A345
- gen-m9/a5828d3925575713.case.json (7): Calc!A10, A84, A102, A194, A314, A332, A360
- gen-m9/a612e89321a6df9c.case.json (9): Calc!A19, A102, A111, A203, A240, A249, A286, A295, A332
- gen-m9/a61e547a1cab994c.case.json (7): Calc!A58, A72, A194, A241, A255, A316, A377
- gen-m9/a790a4d4a6a82756.case.json (6): Calc!A45, A125, A253, A314, A350, A375
- gen-m9/a7bf0d8d81d33ba6.case.json (6): Calc!A142, A223, A284, A325, A386, A396
- gen-m9/abc2939ebd1e2365.case.json (10): Calc!A14, A128, A152, A174, A220, A266, A312, A336, A358, A382
- gen-m9/b4173fe25109a238.case.json (6): Calc!A104, A165, A195, A226, A287, A317
- gen-m9/b6593d7dd13f2dac.case.json (3): Calc!A39, A245, A344
- gen-m9/b8ee55934c5b824c.case.json (11): Calc!A5, A25, A71, A143, A163, A209, A212, A281, A364, A373, A393
- gen-m9/bbdfc06cae0e0d71.case.json (7): Calc!A56, A80, A102, A126, A172, A218, A264
- gen-m9/bc40d37cb4d8301d.case.json (6): Calc!A149, A153, A332, A359, A393, A397
- gen-m9/bcb24385baafb2f0.case.json (10): Calc!A11, A56, A117, A133, A194, A239, A255, A300, A316, A361
- gen-m9/c18dbf21f11093f3.case.json (10): Calc!A34, A80, A126, A172, A218, A224, A310, A316, A356, A362
- gen-m9/c2974c061384fdd9.case.json (4): Calc!A80, A93, A215, A324
- gen-m9/c468f5046aadfbbc.case.json (5): Calc!A9, A156, A217, A253, A400
- gen-m9/c6cfda6bd10cae93.case.json (3): Calc!A103, A164, A347
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A14, A163
- gen-m9/cde551ea787f713a.case.json (7): Calc!A16, A77, A138, A260, A339, A382, A400
- gen-m9/d8266bc15533702a.case.json (3): Calc!A105, A171, A227
- gen-m9/e6cf3b20a014e6a3.case.json (4): Calc!A56, A69, A117, A267
- gen-m9/eced4bb6a0234c9e.case.json (12): Calc!A28, A82, A120, A128, A166, A174, A212, A220, A258, A266, A350, A396
- gen-m9/ee2497a5e683c5d9.case.json (6): Calc!A110, A156, A202, A294, A340, A385
- gen-m9/f2bd5f380ec002bf.case.json (10): Calc!A13, A26, A31, A72, A118, A164, A256, A348, A381, A394
- gen-m9/f34f66fb7f81e673.case.json (6): Calc!A138, A142, A184, A234, A322, A372
- gen-m9/fd9fa22d212080d6.case.json (9): Calc!A35, A53, A96, A114, A157, A175, A297, A340, A358
- gen-m9/fec39efaba9284c3.case.json (8): Calc!A28, A74, A82, A174, A220, A258, A304, A396

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): F.TEST and FTEST nested in the statistical calls with a scalar or text argument (`WEIBULL(-2,F.TEST(ASIN(0),Data!B3:E12),1,"TRUE")` is `#DIV/0!`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 visibly off.
Examples:
- gen-m9/41248d1cb153c66e Calc!A156 `=PERCENTILE.EXC(ROUND(FTEST(100,(-1)),0),1E-100)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/873f30d2bf76f8db Calc!A79 `=NEGBINOMDIST(0,2,((-1)*FTEST(7,Data!A11)))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/afc00c8b70e78baf Calc!A143 `=NORMINV(1E-300,F.TEST({0,0,0},2),Data!$E8)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/ce312817dc9c7f0b Calc!A159 `=PEARSON(-FTEST(1,1e-15),{0.02,0.027})`: Excel #DIV/0!, ours #VALUE!
- gen-m9/d8f44d556674c2fd Calc!A242 `=WEIBULL((-2),F.TEST(ASIN(0),Data!B3:E12),1,"TRUE")`: Excel #DIV/0!, ours #VALUE!
Cells:
- gen-m9/097af1ca33638670.case.json (2): Calc!A52, A272
- gen-m9/09b47629f54b9817.case.json (1): Calc!A182
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A105
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A70
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A156
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A154
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A243
- gen-m9/873f30d2bf76f8db.case.json (2): Calc!A25, A79
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A30
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A277
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A143
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A292
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A159
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A242
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A260

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 426 known-failure lines (409 from T-2255, 17 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 cell of the ungated probes-numeric suite passes in `make eval`, with no new known-failure lines in any baseline.
- [ ] The five cells above pass in `make eval`. t1049-ftest A15, A16 and A22 still pass, and no
      gated baseline (eval/baseline/*.txt) gains an F line.
- [ ] A unit test covers `F.TEST(3,4)`, `F.TEST("a",{1,2,3})` and `FTEST(1,{1,2,3})`, each `#DIV/0!`.
      The code comment cites the probe cells.
- [ ] statistics.md's T-1049 entry says the scalar rule is implemented (T-2093).
- [ ] `make check` green; reviewers approve.

## Out of scope
- An empty one-cell reference (T-2426) and overflowing data (T-2432).
- F.TEST's last digits (T-1927, T-0970).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` where Excel is `#DIV/0!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
