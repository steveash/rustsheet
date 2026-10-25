---
id: T-2493
title: "NPV at a rate of -1 is #DIV/0!, and NPV's error order (gen-m9, 7 cells; NPER cut to T-2698)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
NPV at a rate of -1 returns Excel's `#DIV/0!`, and NPV gives Excel's error order. (The NPER half, NPER at a rate of
1E-300 `#DIV/0!` and NPER(-1,..) `#NUM!`, was cut to the sonnet task T-2698 on 2026-10-25 with its 7 cells.)

## Context
Gap triage of the T-2257 gen-m9 financial sweep.
- (Moved to T-2698.) `NPER(1E-300,-236,2766,945)` and five more with a 1E-300 rate are `#DIV/0!` in Excel (ours `#NUM!`): Excel takes the
  `log(1+r)` path, where 1+1E-300 is 1, not the zero-rate path.
- `NPV(-1,..)` is `#DIV/0!` (`NPV(-1,GESTEP(-2.5,-100))`, `NPV(-1,Data!E7:E12,Data!B2:B4)`; ours `#NUM!`, 6 cells).
- (Moved to T-2698.) `NPER(-1,-31,1)` is `#NUM!` (ours 0).
- `NPV(<#VALUE!>,Data!D9:D12,{-100})` is `#NUM!` and `NPV(1.0049,0,<#DIV/0!>,<#N/A>)` `#N/A`: their inner calls (WEIBULL/ZTEST,
  TOROW, RANK.EQ) may differ in Excel; probe before changing NPV's order (T-2103 is the NPV error-order probe).
docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `fn_nper` in crates/rustsheet-engine/src/functions/financial/annuity.rs, `fn_npv` in functions/financial/mod.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 7 known-failure lines in eval/baseline/gen-m9.txt, 7 visibly off
(14 before the NPER cells moved to T-2698).
Examples:
- gen-m9/0a957119475760c5 Calc!A157 `=NPV((WEIBULL(Data!$A10,1,Data!$C4,0.5)/ZTEST(Data!A6:D7,Data!C4)),Data!D9:D12,{-100})`: Excel #NUM!, ours #VALUE!
- gen-m9/0a957119475760c5 Calc!A255 `=NPV(((Data!E$12*1.005)+1.005),(SECOND(Data!$C10)*COT(1.005)),(TOROW(Data!A8:B11)/(Data!B7+Data!B8)),RANK.EQ(CEILING(2.5,1000000000000000.0),Data!D2:D5,1))`: Excel #N/A, ours #DIV/0!
- gen-m9/3aa168259335be54 Calc!A18 `=NPV((-1),GESTEP((-2.5),-100))`: Excel #DIV/0!, ours #NUM!
- gen-m9/3c06f9298f65a3ba Calc!A50 `=NPER((-1),-31,(Data!C$1^YEAR(0.1)))`: Excel #NUM!, ours 0.0
- gen-m9/3c06f9298f65a3ba Calc!A148 `=NPER(1E-300,-236,2766,945)`: Excel #DIV/0!, ours #NUM!
Cells:
- gen-m9/0a957119475760c5.case.json (2): Calc!A157, A255
- gen-m9/3aa168259335be54.case.json (1): Calc!A18
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A138
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A97
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A192
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A283

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `NPV(-1,{-100,10,20,30,40},<blank>)` and `NPV(-1,Data!E10:E11,..)` are `#DIV/0!` (ours `#NUM!`), and `NPV(1E+300,{0,0,-100,50,60})` is `#DIV/0!` too (ours 0).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/60b0b6e69ad35a45 Calc!A69 `=BETADIST(100,(NPV((-1),{-100,10,20,30,40},Data!C1)+MEDIAN(Data!E4:E5,(-1))),0.999)`: Excel #DIV/0!, ours #NUM!; inner `NPV((-1),{-100,10,20,30,40},Data!C1)` Excel #DIV/0!, ours #NUM!
- gen-m9/f2bd5f380ec002bf Calc!A398 `=HYPGEOM.DIST(3,3,(NPV((-1),Data!E10:E11,Data!A7:A8,Data!A$9)^(Data!B7*Data!C8)),19,(LOOKUP(Data!A12,Data!B2:B2)<LOGNORM.DIST(Data!B9,Data!C12,Data!A12,"x")))`: Excel #DIV/0!, ours #NUM!; inner `NPV((-1),Data!E10:E11,Data!A7:A8,Data!A$9)` Excel #DIV/0!, ours #NUM!
- gen-m9/c468f5046aadfbbc Calc!A378 `=GAMMAINV(Data!D$9%,GAMMALN(ROWS(Data!B10:E12)),NPV(1E+300,{0,0,-100,50,60})%)`: Excel #DIV/0!, ours #NUM!; inner `NPV(1E+300,{0,0,-100,50,60})` Excel #DIV/0!, ours 0.0
Cells:
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A69
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A398
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A378

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 2 gen-m9 financial cells moved here from T-2496. NPV at 1E+300 is `#DIV/0!` too (financial.md "Financial cells of the T-2436 sub-expression probe").

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/b6a02893a8d6b151 Calc!A78 | `NPV((-1),1e-15,Data!C3:C3)` (inside YIELD's COUPDAYBS) | #DIV/0! | #NUM! |
| gen-m9/1cd3096e3e87c965 Calc!A334 | `NPV(1E+300,Data!D5:D8,Data!D4:D5)` (numbers 7, 1E+15, 2.5; inside COUPDAYS) | #DIV/0! | 6.999999999999999E-300 |

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): these 3 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2257): these 7 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests for NPV at -1 (NPER's are T-2698's).
- [x] Visible parity (gen-m9, T-2496): the 2 known-failure lines of the T-2496 probe block above disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- NPV last digits (T-1528).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed (excel-graph, 2026-10-25) in probes/npv-power-edges (51 cells) and npv-power-edges-2 (48 cells), all passing now:
NPV divides each value by the previous period's power (a double) times 1+rate, so it is `#DIV/0!` when 1+rate is 0 or
when a power a later period builds on overflows or falls below the smallest normal double; the last period's power may
overflow (`NPV(1E+300,1,1E+300)` is 2E-300). An error inside a range or array is the result only when no direct argument
errs. New `excel_npv` (financial_util.rs) and the deferred error in `npv_cash_flows`; financial.md "NPV's error order" and
"NPV's discount past the double range". Eval: gen-m9 known failures 4345 -> 4325 (20 fixed, every owned cell; Calc!A157
already passed on main), probes 2659 -> 2611 (48 fixed, 2 new cases at 0 failures), 0 regressions.
