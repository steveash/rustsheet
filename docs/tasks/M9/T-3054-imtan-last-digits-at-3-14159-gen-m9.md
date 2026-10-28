---
id: T-3054
title: "IMTAN last digits at 3.14159 (gen-m9 756a9f7bc3a06bcc A338, unmasked by the T-3030 CELL re-fill)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`IMTAN("3.14159")` matches Excel's last digits: -2.65358979336212E-06 (gen-m9/756a9f7bc3a06bcc Calc!A338).

## Context
- T-3030 re-filled the gen-m9 case from excel-graph, after Excel for the web began reading `CELL("contents",Data!A12)`
  on Data instead of the formula's own sheet. The cell's IMTAN argument is now "3.14159" (Data!A12) where it used to be
  "4", and ours gives -2.65358979336207E-06. Decision 0153 lets it into eval/baseline/gen-m9.txt as a known failure.
- At this argument tan(x) is near zero, so the result is all cancellation. Excel's IMTAN kernel there (sin/cos ratio,
  or sin 2x / (cos 2x + cosh 2y)) decides the digits. See docs/excel-semantics for the existing IMTAN/IMCOT entries
  (T-3032).

## Acceptance criteria
- [ ] gen-m9/756a9f7bc3a06bcc Calc!A338 passes, and no other IM* cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other complex functions.
