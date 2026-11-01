---
id: T-2469
title: "T.TEST and TTEST: an error sample is that error before tails and type, whose errors come before their range check (gen-m9, 64 cells)"
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
T.TEST and TTEST return Excel's error when a sample is an error value (array1 first, then array2), and an error tails or
type before the tails/type range check; a typed empty text sample is no number (`#DIV/0!`).

## Context
Gap triage of the T-2256 gen-m9 sweep. statistics.md "T.TEST: an error sample first, then tails and type errors, then
their range check (T-2256)" records the rule; 52 of the 64 cells are a direct error sample (the engine checks tails/type
first and gives `#NUM!`), the rest an error tails or type behind an out-of-range one, or a typed text sample. A one-cell
reference to an empty cell is T-2426's (its T.TEST cells are listed there). T-1813's order for an error inside a range
(tails/type first) is unchanged. Two cells are unexplained (`TTEST(CELL("col",Data!D4),Data!A5:B10,2,2)` is `#NUM!`,
`TTEST(IRR(..),Data!B5,2,1)` `#NUM!`): if the rule does not cover them, move them to T-2475 in the PR.
Code: `fn_t_test` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 64 known-failure lines in eval/baseline/gen-m9.txt, 64 visibly off.
Examples:
- gen-m9/73f053591a54bc43 Calc!A280 `=T.TEST({1,2},-CEILING(Data!B8,Data!$C1),Data!C10,1)`: Excel #VALUE!, ours #NUM!
- gen-m9/a5342a7e24fd803e Calc!A15 `=TTEST(CELL("col",Data!D4),Data!A5:B10,2,2)`: Excel #NUM!, ours #DIV/0!
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A31
- gen-m9/09b47629f54b9817.case.json (2): Calc!A296, A372
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A259
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A175
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A137
- gen-m9/201d67f1d59436ed.case.json (2): Calc!A250, A312
- gen-m9/2138b909d8f810ea.case.json (2): Calc!A218, A256
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A353
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A22
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A118
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A220
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A165
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A49
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A59
- gen-m9/44a8841352dc48e9.case.json (2): Calc!A251, A327
- gen-m9/46ac3ef5e61583ca.case.json (2): Calc!A282, A344
- gen-m9/5389dd4d5b5739d2.case.json (2): Calc!A268, A327
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A375
- gen-m9/607e85cfee83a689.case.json (1): Calc!A8
- gen-m9/73f053591a54bc43.case.json (2): Calc!A93, A280
- gen-m9/787ba8d801105a60.case.json (2): Calc!A114, A300
- gen-m9/80586b859b73c87d.case.json (3): Calc!A32, A66, A321
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A254
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A256
- gen-m9/928d08e3e1eecb5f.case.json (3): Calc!A95, A207, A269
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A332
- gen-m9/941c409532603d32.case.json (1): Calc!A118
- gen-m9/a2262ea466fcc142.case.json (3): Calc!A206, A268, A375
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A15
- gen-m9/a63d1c7d6f96c56d.case.json (2): Calc!A33, A343
- gen-m9/acac71a65a5a1d1b.case.json (3): Calc!A59, A76, A178
- gen-m9/b9a32ce9a1fd0d71.case.json (2): Calc!A122, A274
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A338
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A338
- gen-m9/c2684066d8df263e.case.json (2): Calc!A28, A214
- gen-m9/cb61169f4c033d4b.case.json (2): Calc!A66, A124
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A152
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A261
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A301
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A122, A139
- gen-m9/eccd03df42e40006.case.json (2): Calc!A136, A326
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A79

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `TTEST(<0>,Data!A2:D5,ROUND(0.1,9),<#VALUE!>)` is `#VALUE!`: the type argument's error before the tails range check (ours `#NUM!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/a612e89321a6df9c Calc!A22 `=F.INV.RT(0.975,0,TTEST(CEILING.PRECISE(Data!D$9,0.1),Data!A2:D5,ROUND(0.1,9),(Data!C8/Data!D7)))`: Excel #VALUE!, ours #NUM!; inner `TTEST(CEILING.PRECISE(Data!D$9,0.1),Data!A2:D5,ROUND(0.1,9),(Data!C8/Data!D7))` Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A22

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 1 gen-m9 financial cell moved here from T-2496. A blank one-cell sample is #VALUE! before the type check, like an error sample.

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/3c06f9298f65a3ba Calc!A312 | `T.TEST(Data!E11,Data!A6:C12,2,Data!$D8)` (E11 blank, type 123456789.123; inside TBILLEQ) | #VALUE! | #NUM! |


Owned gen-m9 cells (T-2524 trace, 2026-10-26; arguments evaluated by the engine, which agree with Excel): 4 known-failure
lines in eval/baseline/gen-m9.txt, 4 visibly off. Each is an error sample before an out-of-range or error tails/type:
- gen-m9/3b7990bd4e3307ba Calc!A286 `T.TEST(Data!B7:D10,STDEV.S(..),DVARP(..)/(-0.5),1)`: array2 #DIV/0!, tails -16928; Excel #DIV/0!, ours #NUM!
- gen-m9/607e85cfee83a689 Calc!A256 `T.TEST(DMIN(..),Data!B10:E11,..,3)`: array1 #VALUE!; Excel #VALUE!, ours #NUM!
- gen-m9/9e2225ead8a6a07b Calc!A10 `T.TEST(Data!A6:C8,SEQUENCE(..),..,VLOOKUP(..))`: array2 #VALUE!, tails #NUM!, type #REF!; Excel #VALUE!, ours #NUM!
- gen-m9/91e167cd8d7eb652 Calc!A148 `TTEST(Data!C7:D11,PEARSON(..),DSUM(..),NORM.S.DIST(..))`: array2 #N/A, tails and type #VALUE!; Excel #N/A, ours #VALUE!

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2256): these 64 known-failure lines disappear (all but A251 and A214, see Outcome) from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `T.TEST(1/0,A1:A3,3,2)` is `#DIV/0!`, `T.TEST(A1:A3,NA(),1,9)` `#N/A`, `T.TEST(A1:A3,B1:B3,-2.5,"a")`
      `#VALUE!`, `T.TEST("",A1:A4,1,3)` `#DIV/0!`; with D2 = `#N/A`, `T.TEST(D1:D3,A1:A3,3,2)` stays `#NUM!` (T-1813).
- [x] statistics.md's T-2256 T.TEST entry gets an "Ours:" line naming this task.
- [x] Visible parity (gen-m9, T-2496): the known-failure line of the T-2496 probe block above disappears from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- T.TEST's numeric accuracy (T-0969).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or text (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.

## Also owns (from T-2426)
Blank one-cell reference cells that need this function's argument order, left failing by T-2426 (which made the blank
`#VALUE!` in the shared readers): 583ca2baea0bb30a A389, c2684066d8df263e A90, 0620505280d2ea6e A139, 91e167cd8d7eb652 A24, d8f44d556674c2fd A329.

## Outcome
Most owned cells already passed on `main` (the error-sample-first order was built under T-2475, T-2496 and T-2524); the
one cell left that was this function's was `TTEST(IFERROR(#VALUE!,""),..,1,3)` (46ac3ef5e61583ca A344): T.TEST read a text
sample as `#VALUE!`, now an array of one cell with no number (`#DIV/0!`), with unit tests for the four acceptance formulas.
Two other owned cells stay failing for a cause outside T.TEST: 44a8841352dc48e9 A251 (`FORECAST.ETS.SEASONALITY` is
`#N/IMPL!`, now listed in T-2424) and c2684066d8df263e A214 (`PIVOTBY` array2, ours 0.3720 against Excel's 0.3685; now T-2854).

A typed or computed boolean sample takes the same path as text (shared with COVARIANCE.P/.S) and is unprobed for T.TEST; statistics.md records it as UNVERIFIED.
