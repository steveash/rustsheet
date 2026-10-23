---
id: T-2398
title: "SCAN over a GROUPBY result is one cell (#N/A, 0 or the error), not a scan of its cells"
milestone: M7
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
SCAN over a GROUPBY result gives Excel's single value (`#N/A`, 0 or the group's error), not a scan of each cell.

## Context
Re-split from T-2359. probes/groupby-lifted-args (excel-graph, 2026-10-22):
- `SCAN(GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,w)),0,MIN)` is one cell `#N/A` (A153, ERROR.TYPE 7 in I153); ours is `#VALUE!`.
- `SCAN(GROUPBY(..,SUM),0,MIN)` is one cell 0 (A161).
- `SCAN(GROUPBY(..,LAMBDA(w,1/0)),0,MIN)` is one cell `#DIV/0!` (A169).
- Owned gen-m7 cell (1 line in eval/baseline/gen-m7.txt): gen-m7/b24ce74f08d3f91a.case.json A1850,
  `=SCAN(_xlfn.GROUPBY(Data!C1:D12,Data!B1:D12,LAMBDA(y,y)),Data!E3,_xleta.MIN)`. Excel `#N/A`, ours `#VALUE!`.
- The 1x1 shape suggests SCAN does not receive the GROUPBY's array. Probe what it does receive: compare SCAN over
  `GROUPBY(...)` with SCAN over `INDEX(GROUPBY(...),1,2)`, `TAKE(GROUPBY(...),1)` and `LET(g,GROUPBY(...),g)`, and REDUCE and MAP
  over the same. Code: functions/logical (SCAN), functions/lookup_and_reference/groupby.rs.

## Acceptance criteria
- [ ] A probe case answers what SCAN receives, and arrays.md records the rule.
- [ ] A153, A161, A169 and the gen-m7 cell pass with 0 regressions; unit tests cover the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY's own result.

## Progress (opus run, 2026-10-23)
Probe committed: probes/scan-over-groupby (69 formulas, excel-graph, 2026-10-23). Data!A1:B4 = a,1; b,2; a,3; c,4. Findings:
- SCAN over a GROUPBY call is always one cell, whatever the GROUPBY function:
  - `SCAN(GROUPBY(..,SUM),0,MIN)` is 0.
  - A LAMBDA that ignores the accumulator gives one value: `LAMBDA(a,v,TYPE(v))` is 1, and `ROWS(v)*100+COLUMNS(v)` is 101.
  - One that uses it, `LAMBDA(a,v,a+1)` or a concat, is `#CALC!` (ERROR.TYPE 14). LET, TAKE and VSTACK wrappers
    give the same `#CALC!`.
  - MAP over the same GROUPBY is a normal 4x2, and REDUCE visits all 8 elements.
- Ours already matches all of that except A36: `SCAN(GROUPBY(..,LAMBDA(w,w)),0,MIN)` is `#N/A` in Excel and `#VALUE!` in ours.
- The likely cause is A68:A72. With LAMBDA(w,w), each GROUPBY value cell holds the group's array, and INDEX returns
  it: `INDEX(GROUPBY(..,LAMBDA(w,w)),4,2)` spills the Total row's {1;2;3;4} (A69:A72), and (1,2) spills too (A68).
  Ours gives `#VALUE!` there. So SCAN MIN in Excel meets a nested array, and its `#N/A` probably comes from that, not
  from SCAN itself.
Next: decide whether fixing A36 needs GROUPBY to keep nested arrays in its cells (listed as out of scope here). If so,
split that into its own task and make this one depend on it. Then fix A36, A153, A161, A169 and the gen-m7 cell.
