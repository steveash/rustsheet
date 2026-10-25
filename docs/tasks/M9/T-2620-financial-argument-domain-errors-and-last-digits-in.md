---
id: T-2620
title: "Financial argument-domain errors and last digits in the T-2475 probe: PV, NPER, PRICEDISC, DB, ACCRINT, ACCRINTM, YIELDDISC, YIELD, PMT, PPMT"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
The owned financial calls give Excel's error (the argument-domain check Excel runs first) or Excel's last digits.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (12; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/11eb3ab4f4f8104c.case.json Calc!A109: probe P!A39 `PV(-D03!B$1,0,-713)`: Excel "#6", ours "#2"
- gen-m9/11eb3ab4f4f8104c.case.json Calc!A160: probe P!A45 `PRICEDISC(D03!$A4,D03!D2,0.3,103,(-2.5))`: Excel "#6", ours "#3"
- gen-m9/1b31092a0d79e5f1.case.json Calc!A260: probe P!A92 `NPER(1E-300,-113,3037,368,1)`: Excel "#2", ours "#6"
- gen-m9/31f09e6c48ca245c.case.json Calc!A232: probe P!A136 `PV((-1+2^-52),165,-978,224,0)`: Excel "#2", ours "#6"
- gen-m9/38e4fc309fe199ea.case.json Calc!A89: probe P!A165 `DB(15779,385,37,D11!$B7,D11!C11)`: Excel "#6", ours "#3"
- gen-m9/44a8841352dc48e9.case.json Calc!A7: probe P!A198 `ACCRINT(34443,36178,38986,0.05/12,3587,1,5,"true")`: Excel "#3", ours "#6"
- gen-m9/44a8841352dc48e9.case.json Calc!A291: probe P!A207 `YIELDDISC(36531,45775,97,RANK.EQ(D14!A9,D14!B9:E11,0),(-1))`: Excel "#6", ours "#7"
- gen-m9/44a8841352dc48e9.case.json Calc!A318: probe P!A214 `YIELD(39681,45662,D14!B$8,3.14159,102,4)`: Excel "#6", ours 19.040243161919385
- gen-m9/bcb1105c61a2649d.case.json Calc!A97: probe P!A462 `PMT(D37!C9,346,96301,42,1)`: Excel -278.44797687866065, ours -278.4479768786607
- gen-m9/cb61169f4c033d4b.case.json Calc!A272: probe P!A517 `DB(79665,(-100),17,9)`: Excel "#6", ours #NUM!
- gen-m9/d910f000b60e2eac.case.json Calc!A20: probe P!A543 `PPMT(100,5,217,4143,669,1)`: Excel 0.0, ours 4.768374219565885e-5
- gen-m9/f82473275380882e.case.json Calc!A131: probe P!A567 `ACCRINTM(32899,36547,D48!C9,2035,3)`: Excel "#3", ours "#6"

## Acceptance criteria
- [ ] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).
