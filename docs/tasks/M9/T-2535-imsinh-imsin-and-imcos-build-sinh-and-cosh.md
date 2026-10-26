---
id: T-2535
title: "IMSINH, IMSIN and IMCOS build sinh and cosh as (EXP(y) -/+ EXP(-y))/2 (gen-m9, 38 cells)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-2534]
campaign: visible
---

## Goal
IMSINH, IMSIN and IMCOS give Excel's digits by building sinh and cosh as (EXP(t) - EXP(-t))/2 and (EXP(t) + EXP(-t))/2 with Excel's EXP for every t, including small t where that cancels.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"). `IMSINH("4")` is 27.2899171971278 in Excel (33 cells): (EXP(4)-EXP(-4))/2 with `excel_exp` is 27.289917197127753, which prints that; libm sinh gives 27.28991719712775, printed ...277 (ours). `IMSEC(IMSINH("4"))` -1.80717748625735 follows from the 15-digit text. `IMCOS("1.5E+2-2E-3i")` has imaginary part -0.0014297538124271 = -SIN(150)*(EXP(-0.002)-EXP(0.002))/2 (sinh gives -0.00142975381242709, ours). `IMSIN("1E-200+1E-200i")` is "1E-200" (3 cells): cos(x)*sinh(1E-200) is 0 because EXP(1E-200) = EXP(-1E-200); ours "1E-200+1E-200i". The worksheet SINH uses expm1 below 1 (numbers.md "SINH / COSH / TANH are exp-based"); the IM* functions do not. Check IMCOSH and IMCSCH/IMSECH/IMCOTH-like users of the same parts (COMPLEXs.xlsx, probes eng-im-overflow-edges) for regressions; near |t| = 710 T-2231 suspects a scaled exp, so keep its cells as they are or better. Depends on T-2534 (same functions, the sin/cos half).
Code: `imsin`, `imcos`, `imsinh` (and `imcosh` if it shares the parts) in complex.rs; `excel_exp` in x87_exp.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 38 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas, 38 visibly off.
Examples:
- gen-m9/09e6a8028f5acfae Calc!A89 `=IMSINH("4")`: Excel "27.2899171971278", ours "27.2899171971277"
- gen-m9/1068c44afcd56d53 Calc!A311 `=IMSEC(IMSINH("4"))`: Excel "-1.80717748625735", ours "-1.80717748625762"
- gen-m9/76b07b57099ccc8d Calc!A219 `=IMSIN("1E-200+1E-200i")`: Excel "1E-200", ours "1E-200+1E-200i"
- gen-m9/9fa170a9f17624fd Calc!A190 `=IMCOS("1.5E+2-2E-3i")`: Excel "0.699252204980454-0.0014297538124271i", ours "0.699252204980454-0.00142975381242709i"
Cells:
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A89
- gen-m9/1068c44afcd56d53.case.json (1): Calc!A311
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A37
- gen-m9/3cf48801617a72ad.case.json (1): Calc!A352
- gen-m9/59a16b5ab566ee5c.case.json (2): Calc!A9, A150
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A332
- gen-m9/5bdef392d16a5efb.case.json (1): Calc!A329
- gen-m9/62c886e21a1042af.case.json (1): Calc!A289
- gen-m9/65dc4e4da669f98a.case.json (3): Calc!A132, A179, A320
- gen-m9/65ff9a692665eba9.case.json (2): Calc!A34, A328
- gen-m9/6d16afd40600302b.case.json (1): Calc!A84
- gen-m9/756a9f7bc3a06bcc.case.json (1): Calc!A183
- gen-m9/76551d6fbafc7007.case.json (1): Calc!A222
- gen-m9/76b07b57099ccc8d.case.json (2): Calc!A219, A227
- gen-m9/7820ebc42c7a56a6.case.json (2): Calc!A28, A169
- gen-m9/7c7e5bff12d93365.case.json (1): Calc!A62
- gen-m9/7dc52cfcbc8b4448.case.json (2): Calc!A148, A289
- gen-m9/8c018ae063ac131b.case.json (1): Calc!A17
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A330
- gen-m9/9fa170a9f17624fd.case.json (3): Calc!A62, A160, A190
- gen-m9/a333e1febf9e93cd.case.json (1): Calc!A249
- gen-m9/a7fda34502ed3030.case.json (1): Calc!A362
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A153
- gen-m9/ca182d96dedee546.case.json (1): Calc!A117
- gen-m9/df083d85b436b49a.case.json (2): Calc!A148, A289
- gen-m9/ef33be4fd2389f3c.case.json (1): Calc!A369
- gen-m9/fb73d8163ad87fc6.case.json (2): Calc!A278, A366

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 38 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for IMSINH("4"), IMCOS("150-0.002i") and IMSIN("1E-200+1E-200i").
- [ ] `make check` green; reviewers approve.

## Out of scope
- The overflow edge near 710 (T-2231).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
