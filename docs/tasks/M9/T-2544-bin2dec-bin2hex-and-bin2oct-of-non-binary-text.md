---
id: T-2544
title: "BIN2DEC, BIN2HEX and BIN2OCT of non-binary text are #NUM!, not #VALUE! (gen-m9, 7 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
BIN2DEC, BIN2HEX and BIN2OCT of text that is not binary are `#NUM!`, as Excel gives and as OCT2* and HEX2* already are.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "Base conversions"). Excel: `BIN2DEC("ABC")`, `BIN2HEX("a")`, `BIN2HEX(" x ")`, `BIN2HEX("TRUE",3)`, `BIN2OCT("abc")`, `BIN2OCT("abc",4)` and `BIN2OCT("a*",7)` (text cells) are `#NUM!`; ours `#VALUE!`. Ours already gives `#NUM!` for `BIN2DEC("2")`, `OCT2DEC("abc")` and `HEX2DEC("xyz")`. A boolean stays `#VALUE!` (`BIN2DEC(TRUE)`, engineering.md "Engineering function argument modes").
Code: `binary_digits`/`base_digits` in crates/rustsheet-engine/src/functions/engineering/number_basis.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 7 known-failure lines in eval/baseline/gen-m9.txt, 7 formulas, 7 visibly off.
Examples:
- gen-m9/7096c4cd615d43ac Calc!A281 `=BIN2OCT(Data!A$12,4)`: Excel #NUM!, ours #VALUE!
- gen-m9/756a9f7bc3a06bcc Calc!A73 `=BIN2DEC(Data!$E$8)`: Excel #NUM!, ours #VALUE!
- gen-m9/a1b4181c04804efc Calc!A316 `=BIN2HEX(Data!$A12)`: Excel #NUM!, ours #VALUE!
- gen-m9/a333e1febf9e93cd Calc!A289 `=BIN2OCT(Data!D10,7)`: Excel #NUM!, ours #VALUE!
- gen-m9/bf979a17ab5aafcb Calc!A331 `=BIN2HEX(Data!D1,3)`: Excel #NUM!, ours #VALUE!
- gen-m9/e882060cf4613302 Calc!A155 `=BIN2OCT(Data!E2)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/7096c4cd615d43ac.case.json (1): Calc!A281
- gen-m9/756a9f7bc3a06bcc.case.json (1): Calc!A73
- gen-m9/a1b4181c04804efc.case.json (1): Calc!A316
- gen-m9/a333e1febf9e93cd.case.json (1): Calc!A289
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A331
- gen-m9/e882060cf4613302.case.json (1): Calc!A155
- gen-m9/fb73d8163ad87fc6.case.json (1): Calc!A11

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `BIN2DEC("Hello World")` and `BIN2HEX("#N/A",1)` (text cells) are `#NUM!` (ours `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/35e8168aae2b00f8 Calc!A108 `=GAMMALN.PRECISE((BIN2DEC(Data!$D$6)*7))`: Excel #NUM!, ours #VALUE!; inner `BIN2DEC(Data!$D$6)` Excel #NUM!, ours #VALUE!
- gen-m9/4ab2ba98d26c8a5e Calc!A116 `=HARMEAN(LET(w,RIGHT(Data!$E6,0),BIN2HEX(Data!C10,1)),Data!C2:C4,(PEARSON(Data!E3:E3,(-0.5))*DB(27087,739,39,Data!D$6,9)))`: Excel #NUM!, ours #VALUE!; inner `LET(w,RIGHT(Data!$E6,0),BIN2HEX(Data!C10,1))` Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A108
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A116

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 1 gen-m9 financial cell moved here from T-2496.

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/d0854366830dcdee Calc!A137 | `BIN2DEC(Data!A6)` (A6 = "x"; inside TBILLPRICE's SUM) | #NUM! | #VALUE! |

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2258): these 7 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for BIN2DEC, BIN2HEX and BIN2OCT of "abc" and " x ".
- [ ] Visible parity (gen-m9, T-2496): the known-failure line of the T-2496 probe block above disappears from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
BIN2DEC/BIN2HEX/BIN2OCT of non-numeric text now give `#NUM!` (booleans and DEC2* text stay `#VALUE!`); unit tests added. Eval: gen-m9 known failures 5367 -> 5356 (11 fixed), probes 2539 -> 2524 (15 fixed), no regressions.
