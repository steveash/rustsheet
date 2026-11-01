---
id: T-3251
title: "Typed minus-dollar mixed fraction with percent must stay text, not #ERROR!"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `-$3 1/2%` stays a text cell, as in Excel, instead of evaluating to `#ERROR!`.

## Context
probes/typed-mixed-fraction-number-format A15/B15 (excel-graph): Excel keeps the text `-$3 1/2%` (`=A15+0` is `#VALUE!`); ours shows `#ERROR!`, so the typed text starting with `-` seems to reach the formula path.

Gap triage 2026-11-01 (`make eval` on origin/main 408444f20): ours now leaves A15 empty (no displayed text) and
`=A15+0` is 0, so the typed text is dropped rather than kept. The cell is an M2 exit cell (probes M0-M2 row:
`make exit-owners MILESTONE=M2` lists it), so the task moves to M2.
- probes/typed-mixed-fraction-number-format.case.json (2): Sheet1!A15, B15 Found by T-3237; `mixed_fraction` already rejects the input. Excel still gives that cell the number format `0.00%` (A15 ENTERED numberFormat), so the fix sets the format too.

Milestone: moved from M9 to M2 by gap triage 2026-10-31. The case counts toward the M0-M2 probe row (decision
0051), and `make exit-owners MILESTONE=M2` listed B15 as the only failing M2 exit cell with no open M2 owner
(decision 0096).

## Acceptance criteria
- [ ] A15/B15 of that probe pass (TF/F lines leave eval/baseline/probes.txt).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
