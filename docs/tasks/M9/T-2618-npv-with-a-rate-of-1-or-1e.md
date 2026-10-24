---
id: T-2618
title: "NPV with a rate of -1 or 1E+300 is #DIV/0! (T-2475 probe)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
NPV is `#DIV/0!` when its rate is -1 (every 1/(1+rate)^k divides by 0) and when it is 1E+300 (the powers overflow), as Excel gives, so the owned gen-m9 cells match.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (6; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/1db18890916ee643.case.json Calc!A320: probe P!A116 `NPV((-1),(-0.5),D06!A$9,D06!B8:B11)`: Excel "#2", ours #NUM!
- gen-m9/329b177b5d5165ad.case.json Calc!A157: probe P!A148 `NPV((-1),1.005,D09!$E1,D09!C11)`: Excel "#2", ours #NUM!
- gen-m9/3b7990bd4e3307ba.case.json Calc!A261: probe P!A177 `NPV(((-1)-D12!$C4),D12!B1:B2,D12!B7:B12,(0.5+D12!B5))`: Excel "#2", ours #NUM!
- gen-m9/607e85cfee83a689.case.json Calc!A28: probe P!A236 `NPV((-1),D17!B7:B11)`: Excel "#2", ours #NUM!
- gen-m9/7fd0a98183c6babf.case.json Calc!A240: probe P!A297 `NPV(D22!$B4,D22!D8,D22!A5:A5,D22!C6:C9)`: Excel "#2", ours #NUM!
- gen-m9/acac71a65a5a1d1b.case.json Calc!A27: probe P!A429 `NPV(1E+300,(D34!E3+0.5),{-100,-200,-300})`: Excel "#2", ours 8e-301

## Acceptance criteria
- [ ] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).
