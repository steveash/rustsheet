---
id: T-3537
title: "NETWORKDAYS.INTL off by one with a fractional or serial-0 start or end (gen-m6 arg-pairs cells)"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
NETWORKDAYS.INTL matches Excel's count when the start or end is fractional or serial 0 with a weekend code of 3, 13, 14 or 15.

## Context
Found by the T-3495 arg-pairs batch (oracle excel-graph): the engine is off by one in six cells. Probe the rule (excel-probe skill; see dates.md on serial 0 and fractional serials) before changing code and record it in docs/excel-semantics/dates.md.

## Acceptance criteria
- [ ] The six gen-m6 cells below match Excel.
- [ ] `make eval-baseline` shrinks gen-m6; `make check` green; reviewers approve.

## Out of scope
- Other functions; the unrelated cells of these cases.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error or a value off in a leading digit). The acceptance criteria are the stage.
Cells:
- 102b2df13f6891d2 A547
- 1b4963f7cb42e2fa A385
- ade135f51e0b7a36 A31
- ade135f51e0b7a36 A385
- b91dcfba0176956e A493
- b91dcfba0176956e A499
