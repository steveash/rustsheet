---
id: T-2495
title: "ISPMT, DB, DDB, SYD, PPMT, CUMIPMT, CUMPRINC, DOLLARDE, DOLLARFR and NOMINAL: argument error order (gen-m9, 13 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
ISPMT, DB, DDB, SYD, PPMT, CUMIPMT, CUMPRINC, DOLLARDE, DOLLARFR and NOMINAL return Excel's error when two of their arguments are invalid in different ways.

## Context
Gap triage of the T-2257 gen-m9 financial sweep: single cells in which two arguments disagree, for example
`DB(-0.659,<#VALUE!>,39,1)` and `SYD(<#VALUE!>,1.48,TRUE,9)` are `#NUM!` (ours `#VALUE!`), `ISPMT(1E+10,2,341,<#NUM!>)` is
`#VALUE!`, `DDB(0,<#N/A>,13,8)` `#VALUE!`, `DOLLARFR(<#NUM!>,-0.0001)` `#DIV/0!`, `NOMINAL(-35125,<#VALUE!>)` `#NUM!`. Each
cell's other argument nests a statistical or database call (RSQ, DGET, BETA.DIST, PERCENTILE.INC, COVARIANCE.P, T.TEST,
HYPGEOMDIST, MDETERM) whose Excel value may be what differs: probe each bare call with the case's Data sheet (excel-probe
skill) first, fix the outer order only where the probe shows the inner value is Excel's, and move the rest to T-2496 or
the inner function's task. docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: crates/rustsheet-engine/src/functions/financial/annuity.rs, financial_depreciation.rs, cumulative.rs,
rate_conversions.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 13 known-failure lines in eval/baseline/gen-m9.txt, 13 visibly off.
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A240 `=DOLLARDE(RSQ(Data!A3:D12,Data!E8),23)`: Excel #VALUE!, ours #N/A
- gen-m9/30e20ba3302fea8b Calc!A174 `=ISPMT(1E+10,2,341,DGET(Db!$A$1:$D$9,2,Data!C9:D9))`: Excel #VALUE!, ours #NUM!
- gen-m9/5c6a23c4f6597f1c Calc!A9 `=DOLLARFR(NPV((-1),Data!E8),(Data!A2-CHISQ.INV(Data!B12,3)))`: Excel #DIV/0!, ours #NUM!
- gen-m9/79d87fc527927626 Calc!A247 `=DB((EXP(0.5)/(-2.5)),PERCENTILE.INC(Data!D2,(-0.5))%,39,1)`: Excel #NUM!, ours #VALUE!
- gen-m9/7d2979fba2c95d64 Calc!A13 `=DDB(0,((123456789.123*(-1))^COVARIANCE.P(Data!B4:C10,Data!C3)),13,8)`: Excel #VALUE!, ours #N/A
- gen-m9/b72d952c5f52cb4d Calc!A100 `=SYD(DGET(Db!A2:D9,2,Db!R1:R2),98,35,2)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A240
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A174
- gen-m9/3aa168259335be54.case.json (1): Calc!A108
- gen-m9/5422bdbf173fc233.case.json (1): Calc!A29
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A9
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A196
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A352
- gen-m9/79d87fc527927626.case.json (2): Calc!A231, A247
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A13
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A100
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A104
- gen-m9/d0854366830dcdee.case.json (1): Calc!A82

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 13 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Each cell is fixed, or moved to a named task in the Outcome with its probe result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
