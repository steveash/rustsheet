---
id: T-2621
title: "Statistical argument-domain errors and extreme values in the T-2475 probe: PERMUTATIONA, COMBINA, T.TEST, T.INV.2T, PERCENTRANK, TRIMMEAN, MODE.MULT, STEYX, SLOPE, distributions"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
The owned statistical calls give Excel's error for out-of-domain arguments, or its value at extreme arguments.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (23; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/097af1ca33638670.case.json Calc!A358: probe P!A28 `PERMUTATIONA(D02!D$12,D02!A12)`: Excel 1.0, ours "#6"
- gen-m9/11eb3ab4f4f8104c.case.json Calc!A90: probe P!A35 `COMBINA((-0.5),D03!$A3)`: Excel 1.0, ours "#6"
- gen-m9/11eb3ab4f4f8104c.case.json Calc!A169: probe P!A58 `T.TEST((D03!A11+(-0.5)),(42/D03!$D6),1,(-1))`: Excel "#3", ours "#6"
- gen-m9/1b31092a0d79e5f1.case.json Calc!A171: probe P!A75 `PERCENTRANK.INC(D05!A7:D8,2,-1e-15)`: Excel "#7", ours "#6"
- gen-m9/1b31092a0d79e5f1.case.json Calc!A179: probe P!A78 `WEIBULL.DIST(IRR({-100,100}),1E+8,NORMDIST(40,(D05!A11*1),MAX(D05!C1:C8,D05!C8:C10),(D05!$A$9<D05!B$1)),(D05!D11<=GAMMA(`: Excel 0.0, ours "#6"
- gen-m9/1b31092a0d79e5f1.case.json Calc!A343: probe P!A96 `MODE.MULT(D05!A8:A8,D05!C7:C10,D05!D3:D11)`: Excel "#3", ours "64|2x1|2.5;1.005"
- gen-m9/2138b909d8f810ea.case.json Calc!A344: probe P!A124 `T.INV.2T(D07!$A4,1)`: Excel "#2", ours "#6"
- gen-m9/2138b909d8f810ea.case.json Calc!A358: probe P!A127 `BETA.INV(0.025,1e-15,2.5)`: Excel "#6", ours 2.518020676354722e-31
- gen-m9/334f3dc9ca937f10.case.json Calc!A226: probe P!A158 `STDEV.P(D10!A9:A12,IFERROR(1/0,D10!$E6),{1,2,3,4,5,6,7,8,9,10})`: Excel 25.559768683380703, ours 26.354817185647274
- gen-m9/5919660b7bb4d195.case.json Calc!A88: probe P!A216 `POISSON.DIST(((D15!E12+0.2)*(1000000000000000.0*1000000000000000.0)),((D15!D5-D15!D10)+HOUR(D15!A9)),(-1))`: Excel 1.0, ours "#6"
- gen-m9/5cebac1f02bf30f2.case.json Calc!A311: probe P!A222 `TTEST(((D16!C6+D16!E$6)/FINV(1.5,3,0.3)),IRR(INDEX(D16!D3:E9,D16!$C7),ROUNDDOWN(1,42)),2,(INDEX(D16!E11:E11,0.5,2.5)^TBI`: Excel "#3", ours "#4"
- gen-m9/6e18550e9fc9c1b9.case.json Calc!A347: probe P!A245 `SKEW({1E+200,1E+200,-1E+200},2.5,D18!D5)`: Excel "#2", ours 0.0
- gen-m9/775d053de498b8c7.case.json Calc!A98: probe P!A266 `STEYX(ERROR.TYPE(SUMPRODUCT(D20!C1:E5,7)),MATCH((D20!$E$8>=D20!$E7),D20!A11:C11))`: Excel "#6", ours "#2"
- gen-m9/775d053de498b8c7.case.json Calc!A112: probe P!A273 `SLOPE(CELL("row",D20!E3),(0.1-TIME(5,16,59)))`: Excel "#6", ours "#2"
- gen-m9/873f30d2bf76f8db.case.json Calc!A308: probe P!A317 `F.INV.RT(1-1E-10,8,7)`: Excel 0.0012599666259138292, ours 0.0012599666259138298
- gen-m9/9bc247733a1ca9fa.case.json Calc!A5: probe P!A341 `BINOM.DIST(3,1E+15,1E-20,FALSE)`: Excel "#6", ours 1.6666500000833278e-16
- gen-m9/9bc247733a1ca9fa.case.json Calc!A360: probe P!A353 `TRIMMEAN(D28!A6,D28!C2)`: Excel "#6", ours "#3"
- gen-m9/a4090567758520df.case.json Calc!A101: probe P!A396 `PERCENTRANK.EXC(D31!A9:C9,D31!B12,D31!B$3)`: Excel "#7", ours "#6"
- gen-m9/a5342a7e24fd803e.case.json Calc!A166: probe P!A407 `WEIBULL(D32!E2,D32!E2,D32!$E11,D32!$A$8)`: Excel "#6", ours 1.0
- gen-m9/a63d1c7d6f96c56d.case.json Calc!A92: probe P!A417 `NEGBINOMDIST(7,9,1E-10)`: Excel 6.434999995495541e-87, ours 6.434999995495502e-87
- gen-m9/bbab6562e88b70e8.case.json Calc!A290: probe P!A455 `PERCENTRANK(D36!C11:E12,(-0.0001),1000000000000000.0)`: Excel "#2", ours #NUM!
- gen-m9/d208bbafc3bc1997.case.json Calc!A262: probe P!A520 `SKEW(D41!D11:D12,D41!D10)`: Excel 2.4980018054066022e-15, ours 2.9976021664879227e-15
- gen-m9/f82473275380882e.case.json Calc!A143: probe P!A572 `NORMDIST(1E+300,MAX(D48!C11:C12),2.5,0)`: Excel "#6", ours 0.0

## Acceptance criteria
- [x] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).

## Handoff (resumed)
Sonnet run (2026-10-26) did not start implementing. The task bundles about 20 functions in one lane, several of
them last-digit numeric fits (F.INV.RT 0.0012599666259138292 vs ...298, NEGBINOMDIST 6.434999995495541e-87 vs ...502,
STDEV.P 25.5597.. vs 26.3548.. with an array third argument) and extreme-argument cutoffs (BINOM.DIST, BETA.INV,
WEIBULL.DIST). That is beyond sonnet's rubric (numeric fitting, unclear evidence). Suggested next steps: take the
pure domain-error groups first (PERMUTATIONA/COMBINA/POISSON.DIST negative or fractional args, T.INV.2T, PERCENTRANK,
TRIMMEAN, STEYX/SLOPE error-type inputs, MODE.MULT), then the numeric ones; consider splitting into two tasks via
`make new-task`.

## Outcome
Split: the three last-digit cells (P!A317 F.INV.RT, P!A417 NEGBINOMDIST, P!A520 SKEW; gen-m9 873f30d2bf76f8db A308,
d208bbafc3bc1997 A262) moved to T-2788 (numeric fitting). Seven owned cells (P!A75, A96, A266, A273, A341, A396, A455)
already passed on main from other tasks. The rest now match Excel: PERMUTATIONA truncates number_chosen (also closes
T-2701); COMBINA truncates number, and number_chosen only when number is 0 (gen-m5/gen-m6 cells show `COMBINA(7,-0.005)` is
`#NUM!`); a first T.TEST sample that is an error value or a blank cell beats bad tails/type; T.INV* at p = 0 with df 1 is
`#DIV/0!`; WEIBULL at x = 0 is 0 and an overflowing (x/beta)^alpha `#NUM!`; POISSON's cdf past a u64 count is 1; IFERROR's
blank fallback is the empty value (STDEV.P counts 0); TRIMMEAN of a lone blank/text/boolean cell is `#NUM!`; SKEW with an
overflowing sum of squares `#DIV/0!`; NORM.DIST's density `#NUM!` where z² overflows; a BETA.INV quantile of 0 or 1 `#NUM!`.
Rules in statistics.md "Statistical argument-domain errors and extreme values (T-2621)" and logic.md. Eval: gen-m9 known
failures 4056 -> 3963 (93 fixed), probes 2848 -> 2816 (32 fixed), no regressions.
