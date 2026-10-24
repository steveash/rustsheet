---
id: T-2580
title: "An empty array (#CALC!) given to SWITCH, IMSUM or WORKDAY's holidays is #VALUE!, and SEQUENCE(0.0123,123456789.123,..) is #VALUE!, not zero rows' #CALC! (gen-m9, 4 cells)"
milestone: M7
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
An empty array (`#CALC!`) passed as a value argument of SWITCH, IMSUM or WORKDAY's holidays is `#VALUE!`, and `SEQUENCE(0.0123,123456789.123,..)` is `#VALUE!`, not zero rows' `#CALC!`, as Excel does (probe SEQUENCE(0.0123,1) against SEQUENCE(1,123456789.123) first: the oversized columns as the cause are UNVERIFIED).

## Context
From T-2436's probe of the untraced gen-m9 cells. arrays.md "An empty array as a function argument (T-2436)". ABS, N and ERROR.TYPE keep `#CALC!`, IFERROR catches it, so the rule is per function: probe the other value-taking functions that accept arrays (CHOOSE value, IFS, XLOOKUP if_not_found, IMPRODUCT, NETWORKDAYS holidays, SUM) in the same PR. Code: SWITCH (functions/logic), IMSUM (functions/engineering/complex), WORKDAY holidays reader (functions/date), SEQUENCE (functions/lookup or array).

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): an empty array (`#CALC!`) as a value argument of SWITCH, IMSUM or WORKDAY's holidays is `#VALUE!`, while ABS, N and ERROR.TYPE keep `#CALC!` (`SWITCH(1,TAKE(0.2,0,3),5,6)`, `SWITCH(DROP({1},1),1,2,3)`, `IMSUM(DROP({1},1))`, `WORKDAY(1,1,DROP({1},1))` `#VALUE!`; probes/t2436/computed-scalar A43:A50; ours `#CALC!`); and `SEQUENCE(0.0123,123456789.123,..)` is `#VALUE!` (not the zero rows' `#CALC!`; the oversized columns as the cause UNVERIFIED; ours `#CALC!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas.
Examples:
- gen-m9/58e3253bbd3a7a82 Calc!A350 `=LOGNORMDIST(DEVSQ({100000000.1,100000000.2,100000000.3},Data!A7:A11,Data!C8:C9),WORKDAY((Data!E$10/2.5),3.14159,SCAN(Data!E$6,{1,#N/A,3},LAMBDA(x,w,{1,2}))),1E+8)`: Excel #VALUE!, ours #CALC!; inner `WORKDAY((Data!E$10/2.5),3.14159,SCAN(Data!E$6,{1,#N/A,3},LAMBDA(x,w,{1,2})))` Excel #VALUE!, ours #CALC!
- gen-m9/a612e89321a6df9c Calc!A390 `=F.INV.RT(SWITCH((Data!D12^Data!E$5),TAKE(0.2,0,3),BITAND(644,670),MOD(Data!D5,Data!E10),Data!B$12,COUNTBLANK(Data!A7:B12),Data!$C6),6,5)`: Excel #VALUE!, ours #CALC!; inner `SWITCH((Data!D12^Data!E$5),TAKE(0.2,0,3),BITAND(644,670),MOD(Data!D5,Data!E10),Data!B$12,COUNTBLANK(Data!A7:B12),Data!$C6)` Excel #VALUE!, ours #CALC!
- gen-m9/5b6cc12a94f7bf14 Calc!A165 `=IMSUM(DROP(Data!A1:C3,2,(Data!B2+Data!E10)))`: Excel #VALUE!, ours #CALC!
- gen-m9/f280df8ba95f96b9 Calc!A222 `=DELTA(SEQUENCE(DISC(38193,44517,85,108),123456789.123,DAY(37033),SKEW(Data!D3:D12,{100000000.1,100000000.2,100000000.3})),SUM(Data!A2:A2,Data!B1:B4))`: Excel #VALUE!, ours #CALC!; inner `SEQUENCE(DISC(38193,44517,85,108),123456789.123,DAY(37033),SKEW(Data!D3:D12,{100000000.1,100000000.2,100000000.3}))` Excel #VALUE!, ours #CALC!
Cells:
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A350
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A390
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A165
- gen-m9/f280df8ba95f96b9.case.json (1): Calc!A222

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.
