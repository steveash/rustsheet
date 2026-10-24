---
id: T-2468
title: "PROB: an error x_range or prob_range is that error, one value on each side is #DIV/0!, unequal counts #N/A (gen-m9, 116 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
PROB returns Excel's result when its data arguments are error values, single values or of different counts: the
x_range error, then the prob_range error, then `#DIV/0!` for one value on each side, then `#N/A` for unequal counts.

## Context
Gap triage of the T-2256 gen-m9 sweep (part 2, M to Z). statistics.md "PROB: an error data argument first, one value on
each side is #DIV/0!, unequal counts #N/A (T-2256)" records the order from these cells (111 top-level PROB cells, four nested in another call); with T-2426's
seven empty-reference PROB cells, 111 of 118 follow it exactly (the other seven: three are the sum-check `#NUM!` for equal counts in different shapes or prob values
that are all booleans, two have an inner YIELD/NPV that differs, two are owned elsewhere). A one-cell reference to an empty
cell is T-2426's `#VALUE!` (its PROB cells are listed there). These are direct error values: an error inside a range keeps
T-1813's order (a bad lower/upper limit first).
Code: `fn_prob` and `numeric_flat_of` in crates/rustsheet-engine/src/functions/statistical/prob.rs (the data argument is
`WholeByCaller`, read once; the limits lift).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 115 known-failure lines in eval/baseline/gen-m9.txt, 115 visibly off.
Examples:
- gen-m9/7fd0a98183c6babf Calc!A235 `=PERCENTILE(Data!D7:D8,(PROB(Data!C6,Data!D6,(-2.5),Data!$A7)+100))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/80586b859b73c87d Calc!A353 `=RANK.EQ(PROB(Data!C9:D9,Data!$D$12,2%,TINV(Data!B11,Data!D5)),Data!C10:C10)`: Excel #N/A, ours #NUM!
- gen-m9/b9a32ce9a1fd0d71 Calc!A103 `=PROB(Data!C5,Data!A7:C9,PI())`: Excel #N/A, ours #VALUE!
- gen-m9/bcb1105c61a2649d Calc!A255 `=PROB(Data!A4:D9,DAY(CRITBINOM(8,1E-20,Data!E8)),Data!E5,((0.5/(-2.5))/NA()))`: Excel #NUM!, ours #VALUE!
- gen-m9/eccd03df42e40006 Calc!A16 `=PROB(((0*3.14159)+DROP(Data!D5,1,Data!C4)),Data!A4:A7,CEILING.MATH(Data!D2))`: Excel #CALC!, ours #VALUE!
Cells:
- gen-m9/0072265d385d8ab2.case.json (2): Calc!A5, A191
- gen-m9/097af1ca33638670.case.json (4): Calc!A100, A162, A224, A286
- gen-m9/09b47629f54b9817.case.json (1): Calc!A62
- gen-m9/0bbab6206b5c9a07.case.json (2): Calc!A74, A136
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A349
- gen-m9/139684e68fb2fc3b.case.json (2): Calc!A49, A201
- gen-m9/17c110a53924beb8.case.json (1): Calc!A330
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A94
- gen-m9/1db18890916ee643.case.json (1): Calc!A388
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A186
- gen-m9/2138b909d8f810ea.case.json (2): Calc!A260, A336
- gen-m9/31f09e6c48ca245c.case.json (2): Calc!A132, A322
- gen-m9/329b177b5d5165ad.case.json (2): Calc!A82, A206
- gen-m9/334f3dc9ca937f10.case.json (2): Calc!A82, A392
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A17
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A235
- gen-m9/3fb9cecf2c23169f.case.json (2): Calc!A12, A384
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A169
- gen-m9/43c6227e0ff368c7.case.json (2): Calc!A137, A323
- gen-m9/44a8841352dc48e9.case.json (2): Calc!A108, A336
- gen-m9/46ac3ef5e61583ca.case.json (2): Calc!A21, A83
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A151
- gen-m9/5395b8a427764a3f.case.json (2): Calc!A82, A268
- gen-m9/583ca2baea0bb30a.case.json (2): Calc!A182, A244
- gen-m9/5919660b7bb4d195.case.json (2): Calc!A153, A343
- gen-m9/5be29fa316931f05.case.json (1): Calc!A215
- gen-m9/5cebac1f02bf30f2.case.json (3): Calc!A30, A68, A372
- gen-m9/607e85cfee83a689.case.json (1): Calc!A275
- gen-m9/6e18550e9fc9c1b9.case.json (3): Calc!A84, A208, A394
- gen-m9/787ba8d801105a60.case.json (3): Calc!A108, A232, A294
- gen-m9/7980ffb6095a340d.case.json (4): Calc!A14, A128, A280, A318
- gen-m9/7f7ca726c3f68554.case.json (2): Calc!A193, A269
- gen-m9/7fd0a98183c6babf.case.json (2): Calc!A65, A235
- gen-m9/80586b859b73c87d.case.json (1): Calc!A353
- gen-m9/873f30d2bf76f8db.case.json (4): Calc!A72, A110, A300, A338
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A127
- gen-m9/928d08e3e1eecb5f.case.json (2): Calc!A307, A369
- gen-m9/92d4fa29e5510f1e.case.json (4): Calc!A32, A70, A184, A374
- gen-m9/941c409532603d32.case.json (3): Calc!A76, A324, A386
- gen-m9/9735c5ab72fa5ecf.case.json (2): Calc!A88, A354
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A176
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A373
- gen-m9/a2262ea466fcc142.case.json (2): Calc!A323, A385
- gen-m9/a4090567758520df.case.json (2): Calc!A73, A135
- gen-m9/a5342a7e24fd803e.case.json (3): Calc!A249, A325, A363
- gen-m9/a63d1c7d6f96c56d.case.json (3): Calc!A5, A191, A315
- gen-m9/b9a32ce9a1fd0d71.case.json (2): Calc!A103, A255
- gen-m9/bcb1105c61a2649d.case.json (2): Calc!A255, A317
- gen-m9/c7664e494e418395.case.json (2): Calc!A39, A74
- gen-m9/ce312817dc9c7f0b.case.json (5): Calc!A78, A116, A154, A230, A382
- gen-m9/d37313406b9f6e55.case.json (3): Calc!A142, A218, A370
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A225, A349
- gen-m9/d8f44d556674c2fd.case.json (2): Calc!A40, A344
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A198
- gen-m9/eccd03df42e40006.case.json (4): Calc!A16, A54, A206, A320
- gen-m9/ede0323c389fc703.case.json (1): Calc!A240
- gen-m9/f82473275380882e.case.json (1): Calc!A21

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `OCT2DEC(ROUNDUP(PROB(<TRUE cell>,-0.0001,2,<empty>),3))` is `#DIV/0!` (one value on each side; ours `#VALUE!`; gen-m9/ee70a5515d004a66 A145).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/ee70a5515d004a66 Calc!A145 `=OCT2DEC(ROUNDUP(PROB(Data!C8,(-0.0001),Data!B9,Data!D10),3))`: Excel #DIV/0!, ours #VALUE!
Cells:
- gen-m9/ee70a5515d004a66.case.json (1): Calc!A145

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): PROB's rule decides each: unequal counts `#N/A` (`PROB(Data!$B$4,Data!D10:E11,1)`, `PROB(<"#N/A" text cell>,Data!B4:E10,0,-2.5)`; ours `#VALUE!`), one value on each side `#DIV/0!` (`PROB(<"TRUE" text>,<"x">,123456789.123,7)`, `PROB(2.5,<TRUE cell>,100,123456789.123)`; ours `#VALUE!`), the data's error first (`PROB(INDEX(Data!A7:B11,4),..)` `#REF!`, ours `#N/A`), and `PROB(Data!D3:D8,Data!C5:C10,0.3,0.3)` over pairs with text and blanks is 0 (ours `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 6 known-failure lines in eval/baseline/gen-m9.txt, 6 formulas.
Examples:
- gen-m9/2801c1c11c21e134 Calc!A132 `=FINV(PROB(Data!$B$4,Data!D10:E11,1),2.5,9)`: Excel #N/A, ours #VALUE!; inner `PROB(Data!$B$4,Data!D10:E11,1)` Excel #N/A, ours #VALUE!
- gen-m9/abc2939ebd1e2365 Calc!A44 `=BETADIST(Data!E1,(-1),PROB(INDEX(Data!A7:B11,4),(Data!C6*(-1)),VLOOKUP("1",Data!B2:C6,Data!$E$7)))`: Excel #REF!, ours #N/A; inner `PROB(INDEX(Data!A7:B11,4),(Data!C6*(-1)),VLOOKUP("1",Data!B2:C6,Data!$E$7))` Excel #REF!, ours #N/A
- gen-m9/c468f5046aadfbbc Calc!A162 `=LOGNORM.DIST(STDEV.P(PROB(Data!B11,Data!B6,Data!$C$9,7),Data!A9:A10),((1000000000000000.0*123456789.123)-NA()),0,FALSE)`: Excel #DIV/0!, ours #VALUE!; inner `PROB(Data!B11,Data!B6,Data!$C$9,7)` Excel #DIV/0!, ours #VALUE!
- gen-m9/e6cf3b20a014e6a3 Calc!A229 `=FINV(1-1E-16,(TRANSPOSE(Data!$B9)*PROB(Data!D3:D8,Data!C5:C10,Data!$C$12,Data!C12)),3)`: Excel #NUM!, ours #N/A; inner `PROB(Data!D3:D8,Data!C5:C10,Data!$C$12,Data!C12)` Excel 0.0, ours #N/A
Cells:
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A132
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A44
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A162
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A229
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A116
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A73

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 6 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2256): these 115 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `PROB(1/0,{0.5;0.5},1)` is `#DIV/0!` and `PROB({1;2},NA(),"a")` `#N/A` (data errors before the limits);
      `PROB(2,0.5,1)` and `PROB(A1,B1,1)` with numbers in A1, B1 are `#DIV/0!`; `PROB(A1:A3,B1:B4,1)` is `#N/A`;
      `PROB(D1:D3,F1:F3,"a")` with D2 = `#N/A` stays `#VALUE!` (T-1813).
- [ ] statistics.md's T-2256 PROB entry gets an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sum-of-probabilities `#NUM!` and the limits' lifting (unchanged).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or text (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.
