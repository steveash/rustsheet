---
id: T-2623
title: "Trace the 33 T-2475 gen-m9 cells the sub-expression probe does not settle"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Each of these gen-m9 cells, whose probed sub-expressions all match Excel in probes/t2475-stat-subexpressions, has its divergence traced (the top call's own argument combination, the copied Data sheet's context, or a sub-expression the probe did not split out) and an owning fix task.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (33; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/097af1ca33638670.case.json Calc!A14
- gen-m9/097af1ca33638670.case.json Calc!A89
- gen-m9/17c110a53924beb8.case.json Calc!A300
- gen-m9/1db18890916ee643.case.json Calc!A89
- gen-m9/334f3dc9ca937f10.case.json Calc!A125
- gen-m9/3b7990bd4e3307ba.case.json Calc!A98
- gen-m9/41248d1cb153c66e.case.json Calc!A394
- gen-m9/6e18550e9fc9c1b9.case.json Calc!A69
- gen-m9/73f053591a54bc43.case.json Calc!A226
- gen-m9/7980ffb6095a340d.case.json Calc!A146
- gen-m9/80586b859b73c87d.case.json Calc!A67
- gen-m9/873f30d2bf76f8db.case.json Calc!A96
- gen-m9/928d08e3e1eecb5f.case.json Calc!A333
- gen-m9/941c409532603d32.case.json Calc!A148
- gen-m9/9bc247733a1ca9fa.case.json Calc!A207
- gen-m9/9e2225ead8a6a07b.case.json Calc!A100
- gen-m9/9e2225ead8a6a07b.case.json Calc!A292
- gen-m9/a2262ea466fcc142.case.json Calc!A316
- gen-m9/a5342a7e24fd803e.case.json Calc!A322
- gen-m9/b9a32ce9a1fd0d71.case.json Calc!A14
- gen-m9/b9a32ce9a1fd0d71.case.json Calc!A142
- gen-m9/b9a32ce9a1fd0d71.case.json Calc!A212
- gen-m9/bcb1105c61a2649d.case.json Calc!A230
- gen-m9/c2684066d8df263e.case.json Calc!A65
- gen-m9/c4635963fc926284.case.json Calc!A12
- gen-m9/c4635963fc926284.case.json Calc!A92
- gen-m9/c4635963fc926284.case.json Calc!A153
- gen-m9/d37313406b9f6e55.case.json Calc!A137
- gen-m9/d37313406b9f6e55.case.json Calc!A182
- gen-m9/d8f44d556674c2fd.case.json Calc!A398
- gen-m9/ede0323c389fc703.case.json Calc!A7
- gen-m9/0620505280d2ea6e.case.json Calc!A4
- gen-m9/a63d1c7d6f96c56d.case.json Calc!A216

The last two read Pivot! through GETPIVOTDATA and were not copied into the probe.

## Acceptance criteria
- [ ] Each cell above is owned by a fix task (new or existing) naming its cause, with the evidence in that task; quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).
