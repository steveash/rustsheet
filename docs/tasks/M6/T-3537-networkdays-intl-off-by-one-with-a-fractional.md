---
id: T-3537
title: "NETWORKDAYS.INTL off by one with a fractional or serial-0 start or end (gen-m6 arg-pairs cells)"
milestone: M6
lane: functions-date
status: done
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
Decision 0083: every cell below is visibly off (the batch's 16 failures are all counted in the visible-parity lock). The acceptance criteria are the stage.
Cells:
- 102b2df13f6891d2 A547
- 1b4963f7cb42e2fa A385
- ade135f51e0b7a36 A31
- ade135f51e0b7a36 A385
- b91dcfba0176956e A493
- b91dcfba0176956e A499

## Outcome
The cause was not the day-0 start or end: a **scalar** holiday of NETWORKDAYS.INTL is dropped only on Saturday/Sunday whatever the weekend argument, while a reference or array holiday follows the weekend mask. Probed with five new probe cases (387 cells, excel-graph) and implemented via `Holidays::scalar` / `is_scalar_node`. The six gen-m6 cells pass; recorded in dates.md.
