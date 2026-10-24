---
id: T-2493
title: "NPER at a rate of 1E-300 and NPV at a rate of -1 are #DIV/0!, and their error order (gen-m9, 14 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
NPER at a rate of 1E-300 and NPV at a rate of -1 return Excel's `#DIV/0!`, NPER(-1,..) Excel's `#NUM!`, and NPV gives Excel's error order.

## Context
Gap triage of the T-2257 gen-m9 financial sweep.
- `NPER(1E-300,-236,2766,945)` and five more with a 1E-300 rate are `#DIV/0!` in Excel (ours `#NUM!`): Excel takes the
  `log(1+r)` path, where 1+1E-300 is 1, not the zero-rate path.
- `NPV(-1,..)` is `#DIV/0!` (`NPV(-1,GESTEP(-2.5,-100))`, `NPV(-1,Data!E7:E12,Data!B2:B4)`; ours `#NUM!`, 6 cells).
- `NPER(-1,-31,1)` is `#NUM!` (ours 0).
- `NPV(<#VALUE!>,Data!D9:D12,{-100})` is `#NUM!` and `NPV(1.0049,0,<#DIV/0!>,<#N/A>)` `#N/A`: their inner calls (WEIBULL/ZTEST,
  TOROW, RANK.EQ) may differ in Excel; probe before changing NPV's order (T-2103 is the NPV error-order probe).
docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `fn_nper` in crates/rustsheet-engine/src/functions/financial/annuity.rs, `fn_npv` in functions/financial/mod.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 14 known-failure lines in eval/baseline/gen-m9.txt, 14 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A157 `=NPV((WEIBULL(Data!$A10,1,Data!$C4,0.5)/ZTEST(Data!A6:D7,Data!C4)),Data!D9:D12,{-100})`: Excel #NUM!, ours #VALUE!
- gen-m9/0a957119475760c5 Calc!A255 `=NPV(((Data!E$12*1.005)+1.005),(SECOND(Data!$C10)*COT(1.005)),(TOROW(Data!A8:B11)/(Data!B7+Data!B8)),RANK.EQ(CEILING(2.5,1000000000000000.0),Data!D2:D5,1))`: Excel #N/A, ours #DIV/0!
- gen-m9/3aa168259335be54 Calc!A18 `=NPV((-1),GESTEP((-2.5),-100))`: Excel #DIV/0!, ours #NUM!
- gen-m9/3c06f9298f65a3ba Calc!A50 `=NPER((-1),-31,(Data!C$1^YEAR(0.1)))`: Excel #NUM!, ours 0.0
- gen-m9/3c06f9298f65a3ba Calc!A148 `=NPER(1E-300,-236,2766,945)`: Excel #DIV/0!, ours #NUM!
Cells:
- gen-m9/0a957119475760c5.case.json (2): Calc!A157, A255
- gen-m9/3aa168259335be54.case.json (2): Calc!A18, A363
- gen-m9/3c06f9298f65a3ba.case.json (2): Calc!A50, A148
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A102
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A138
- gen-m9/a91bad4e84f49739.case.json (2): Calc!A97, A336
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A192
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A43
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A41, A283

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 14 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for NPER at 1E-300 and -1, and NPV at -1.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NPV last digits (T-1528).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
