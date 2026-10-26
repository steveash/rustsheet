---
id: T-2432
title: "KURT and F.TEST over data whose squares overflow (1E+200): Excel's #DIV/0! and #NUM! (gen-m9, 11 cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
KURT and F.TEST over data whose sums of squares overflow (`{1E+200,1E+200,-1E+200}`) return Excel's `#DIV/0!` (KURT) and
`#NUM!` (F.TEST and FTEST) instead of a finite number.

## Context
Gap triage of the T-2255 gen-m9 sweep (the data pool's cancelling large values). statistics.md "F.TEST: an error argument is
that error; overflowing squares are #NUM!, KURT's #DIV/0! (T-2255)" records it: 6 KURT cells (ours -4.9, -13.5, ..), 5
F.TEST/FTEST cells (ours 0). Excel's KURT likely divides by a variance that overflowed to infinity and its F.TEST checks for
an overflowing variance; the exact rule (which intermediate overflows, and whether DEVSQ, SKEW, VAR and STDEV behave
alike on the same data) is to be pinned by a probe of `KURT({1E+200,1E+200,-1E+200})`, `VAR.S(..)`, `DEVSQ(..)`,
`SKEW(..)` and `F.TEST({1,2,3},{1E+200,1E+200,-1E+200})` (excel-probe skill) before the change.
Code: `fn_kurt` (functions/statistical/count_and_average.rs), `fn_f_test` (fisher.rs).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 11 known-failure lines in eval/baseline/gen-m9.txt, 11 formulas.
Examples:
- gen-m9/27601819397dd56e Calc!A246 `=KURT(Data!A1:A12,{1E+200,1E+200,-1E+200},(-0.5))`: Excel #DIV/0!, ours -4.166666666666667
- gen-m9/2a18b2f5a4dc43ac Calc!A130 `=F.TEST(Data!A3:A10,{1E+200,1E+200,-1E+200})`: Excel #NUM!, ours 0.0
Cells:
- gen-m9/27601819397dd56e.case.json (1): Calc!A246
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A130
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A183
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A339
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A120
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A14
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A64
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A186
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A398
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A166, A396

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): SKEW, SKEW.P, Z.TEST and the paired/aggregate calls over `{1E+200,1E+200,-1E+200}`: SKEW and SKEW.P are `#DIV/0!`, Z.TEST `#NUM!`, where ours gives 0 or a number.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 18 known-failure lines in eval/baseline/gen-m9.txt, 18 visibly off.
Examples:
- gen-m9/5cebac1f02bf30f2 Calc!A315 `=SKEW(((Data!D$11-Data!B2)+N("Hello World")),Data!B2:B8,{1E+200,1E+200,-1E+200})`: Excel #DIV/0!, ours 0.0
- gen-m9/928d08e3e1eecb5f Calc!A314 `=SKEW.P({1E+200,1E+200,-1E+200})`: Excel #DIV/0!, ours 0.0
- gen-m9/c4635963fc926284 Calc!A245 `=ZTEST({1E+200,1E+200,-1E+200},(BITLSHIFT(632,Data!$D$2)^INDIRECT("Data!$C$5",Data!$E4)))`: Excel #NUM!, ours 0.5
- gen-m9/d5fd9e05e9e6f466 Calc!A341 `=ZTEST({1E+200,1E+200,-1E+200},COUNT(Data!C3:C6,COLUMN()))`: Excel #NUM!, ours 0.5
- gen-m9/eccd03df42e40006 Calc!A256 `=SKEW({1E+200,1E+200,-1E+200})`: Excel #DIV/0!, ours 0.0
Cells:
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A43
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A100
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A120
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A315
- gen-m9/607e85cfee83a689.case.json (2): Calc!A88, A194
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A314
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A187
- gen-m9/9e2225ead8a6a07b.case.json (2): Calc!A59, A260
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A184
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A268
- gen-m9/c4635963fc926284.case.json (1): Calc!A245
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A289
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A341
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A43
- gen-m9/eccd03df42e40006.case.json (1): Calc!A256
- gen-m9/ede0323c389fc703.case.json (1): Calc!A260

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `SKEW.P({1E+200,1E+200,-1E+200},{1,1,2,2,2,3})` is `#DIV/0!` (ours 0) and `T.TEST(Data!B6:D12,{1E+200,1E+200,-1E+200},1,2)` `#NUM!` (ours 0.5): the overflowing squares.
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/899f9555e67da2d6 Calc!A113 `=EXPON.DIST((-2),(SKEW.P({1E+200,1E+200,-1E+200},{1,1,2,2,2,3})-Data!A1),AND(FALSE,ISBLANK(Data!A$8)))`: Excel #DIV/0!, ours #NUM!; inner `SKEW.P({1E+200,1E+200,-1E+200},{1,1,2,2,2,3})` Excel #DIV/0!, ours 0.0
- gen-m9/a612e89321a6df9c Calc!A375 `=FISHER(T.TEST(Data!B6:D12,{1E+200,1E+200,-1E+200},1,2))`: Excel #NUM!, ours 0.5493061443340549; inner `T.TEST(Data!B6:D12,{1E+200,1E+200,-1E+200},1,2)` Excel #NUM!, ours 0.5
Cells:
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A113
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A375

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 29 known-failure lines (11 from T-2255, 18 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The probe cells are in eval/ and statistics.md's entry has the rule and an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cancellation of finite data (T-1749).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
