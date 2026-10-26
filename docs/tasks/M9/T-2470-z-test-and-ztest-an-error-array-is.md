---
id: T-2470
title: "Z.TEST and ZTEST: an error array is that error, one data value is #DIV/0! before sigma, a lone text or boolean cell is #N/A (gen-m9, 37 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Z.TEST and ZTEST return Excel's results for an error array argument (that error first), a single data value (`#DIV/0!`
before a bad sigma) and a lone single-cell reference holding text, a boolean or nothing (`#N/A`).

## Context
Gap triage of the T-2256 gen-m9 sweep. statistics.md "Z.TEST: an error array first; one data value is #DIV/0!; a lone
non-number cell is #N/A (T-2256)" records the rules from these cells. Data with no number is `#N/A`, whether a range or a
plain single-cell reference holding a non-number (several range cells already pass). Two owned `#DIV/0!` cells over
ranges with no number (7980ffb6095a340d A294, a63d1c7d6f96c56d A324) come from their x or sigma argument, not the
range: trace that sub-expression first (UNVERIFIED). Seven cells nest Z.TEST in another call. Overflowing data (`{1E+200,1E+200,-1E+200}`) is T-2432's.
Code: `fn_z_test` in crates/rustsheet-engine/src/functions/statistical/z_test.rs (x and sigma lift; the data is read once).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 35 known-failure lines in eval/baseline/gen-m9.txt, 35 visibly off.
Examples:
- gen-m9/2138b909d8f810ea Calc!A191 `=ZTEST(SLOPE(Data!B12,-Data!B8),PERMUT(10,100),(TRUNC(Data!D4,1.005)/Data!C3))`: Excel #DIV/0!, ours #NUM!
- gen-m9/31f09e6c48ca245c Calc!A241 `=POISSON.DIST(10,((Data!B8/2)/ZTEST(Data!B11:D11,Data!A8,Data!E$1)),TRUE)`: Excel #DIV/0!, ours #NUM!
- gen-m9/46ac3ef5e61583ca Calc!A133 `=TRIMMEAN(Data!B2:C10,ZTEST(((-2.5)^Data!E3),STEYX(Data!$E3,Data!B10:E10)))`: Excel #VALUE!, ours #N/A
- gen-m9/775d053de498b8c7 Calc!A362 `=T.INV(Data!D$7,PEARSON(ZTEST(Data!B6,1000000000000000.0,0.1),Data!$E12))`: Excel #N/A, ours #VALUE!
- gen-m9/c2684066d8df263e Calc!A190 `=ZTEST(Data!B11,Data!C2)`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/0072265d385d8ab2.case.json (1): Calc!A152
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A231
- gen-m9/1db18890916ee643.case.json (2): Calc!A29, A67
- gen-m9/201d67f1d59436ed.case.json (3): Calc!A99, A223, A347
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A191
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A241
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A242
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A282
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A87
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A320
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A33
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A133
- gen-m9/607e85cfee83a689.case.json (2): Calc!A73, A197
- gen-m9/775d053de498b8c7.case.json (1): Calc!A362
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A294
- gen-m9/7f7ca726c3f68554.case.json (2): Calc!A218, A294
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A54
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A333
- gen-m9/941c409532603d32.case.json (1): Calc!A376
- gen-m9/9735c5ab72fa5ecf.case.json (3): Calc!A83, A158, A272
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A229
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A324
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A4
- gen-m9/c2684066d8df263e.case.json (2): Calc!A190, A223
- gen-m9/c4635963fc926284.case.json (1): Calc!A55
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A332
- gen-m9/ede0323c389fc703.case.json (1): Calc!A307

gen-m9 engineering and database sweep (gap triage 2026-10-23, T-2258): `BITOR(470,ZTEST(0.1,1,-0.0001)-..)` is `#DIV/0!` (one data value before sigma; ours `#NUM!`) and `DMAX(Db!A1:D5,NEGBINOM.DIST(1,7,Z.TEST(<empty cell>,3.14159,1E+8),TRUE),..)` `#N/A` (ours `#VALUE!`).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas, 2 visibly off.
Examples:
- gen-m9/1068c44afcd56d53 Calc!A259 `=BITOR(470,(ZTEST(0.1,1,(-0.0001))-AVERAGE(Data!$E4)))`: Excel #DIV/0!, ours #NUM!
- gen-m9/d9d55972b50e55e8 Calc!A331 `=DMAX(Db!A1:D5,NEGBINOM.DIST(1,7,Z.TEST(Data!B7,Data!B10,1E+8),TRUE()),Db!F1:F3)`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/1068c44afcd56d53.case.json (1): Calc!A259
- gen-m9/d9d55972b50e55e8.case.json (1): Calc!A331

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `Z.TEST(1E+15,Data!$B$3,<blank>)` and `ZTEST(Data!B9:B12,42,<0>)` are `#DIV/0!` (ours `#NUM!`), `ZTEST(<blank>,<blank>,1.5E+10)` and `ZTEST(<"x" cell>,100)` `#N/A` (ours `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas.
Examples:
- gen-m9/58e3253bbd3a7a82 Calc!A46 `=LOGINV(1E-16,INDEX(Data!B10:E12,CEILING.PRECISE(Data!E$7),Z.TEST(1000000000000000.0,Data!$B$3,Data!C4)),INDEX(Data!A8:C8,2,2))`: Excel #DIV/0!, ours #NUM!; inner `Z.TEST(1000000000000000.0,Data!$B$3,Data!C4)` Excel #DIV/0!, ours #NUM!
- gen-m9/a612e89321a6df9c Calc!A241 `=COVAR((ZTEST(Data!D9,Data!$C11,1.5E+10)/2),Data!C1:C12)`: Excel #N/A, ours #VALUE!; inner `ZTEST(Data!D9,Data!$C11,1.5E+10)` Excel #N/A, ours #VALUE!
- gen-m9/a790a4d4a6a82756 Calc!A308 `=CRITBINOM((ZTEST(Data!B9:B12,42,Data!D8)/(0*Data!$C8)),0.05,0.975)`: Excel #DIV/0!, ours #NUM!; inner `ZTEST(Data!B9:B12,42,Data!D8)` Excel #DIV/0!, ours #NUM!
- gen-m9/eced4bb6a0234c9e Calc!A34 `=CONFIDENCE.NORM((ZTEST(Data!$B$2,100)/(Data!D7*1)),99999,14)`: Excel #N/A, ours #VALUE!; inner `ZTEST(Data!$B$2,100)` Excel #N/A, ours #VALUE!
Cells:
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A46
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A241
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A308
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A34

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 2 gen-m9 financial cells moved here from T-2496. The first is a two-cell range with one number (`#DIV/0!`, like `{5}`); the second a lone empty cell (`#N/A`), both as statistics.md "Z.TEST: an error array first" says.

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/5422bdbf173fc233 Calc!A227 | `ZTEST(Data!D7:D8,Data!$A12,Data!$B10)` (D7:D8 = FALSE, 42; one number; sigma FALSE) | #DIV/0! | #NUM! |
| gen-m9/bc04c1f478c9896c Calc!A275 | `ZTEST(Data!$A10,(1.005/2.5),100)` (A10 empty, a lone cell) | #N/A | #VALUE! |

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2258): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] (Partly met; rest in T-2775) Visible parity (gen-m9, T-2256): these 35 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `Z.TEST(1/0,"x")` is `#DIV/0!`, `Z.TEST({5},3,-1)` `#DIV/0!`, `Z.TEST(A1,1)` with A1 = "x" `#N/A`,
      `Z.TEST(A1:A1,1)` with A1 = "x" `#N/A` (amended: statistics.md evidence), and `Z.TEST(A1:A8,4,0)` stays `#NUM!`.
- [x] statistics.md's T-2256 Z.TEST entry gets an "Ours:" line naming this task.
- [x] Visible parity (gen-m9, T-2496): the 2 known-failure lines of the T-2496 probe block above disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- Z.TEST's tail accuracy and the sigma/sqrt(n) underflow (T-1942).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or text (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.

## Outcome
Z.TEST/ZTEST: a one-value data argument is `#DIV/0!` before a sigma of 0 or below, and a plain single-cell
reference holding text, a boolean or nothing is `#N/A`. Eval delta: 24 gen-m9 known-failure lines fixed, no regressions.
Not fixed: the remaining owned cells (nested sub-expressions; A294 and A324 untraced); filed as T-2775. Deviation
from the criteria: the unit test has `Z.TEST(A1:A1,1)` with text as `#N/A`, where the criterion lists `#DIV/0!`; the
statistics.md evidence for `C7:C7` (5919660b7bb4d195 A364) says `#N/A`. The probes.txt lines removed are probe cells
that passed once the one-value order changed (made by `make eval-baseline`).
