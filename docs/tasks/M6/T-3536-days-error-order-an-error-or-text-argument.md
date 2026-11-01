---
id: T-3536
title: "DAYS error order: an error or text argument beats #VALUE! (gen-m6 arg-pairs cells)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DAYS returns the Excel error for an argument pair where an error-valued or error-producing argument meets text, empty text or a blank-derived value.

## Context
Found by the T-3495 arg-pairs batch (oracle excel-graph). Excel returns #NUM!, #NAME? or #DIV/0! where the engine returns #VALUE!. Probe the error order (excel-probe skill) before changing code; record it in docs/excel-semantics/dates.md.

## Acceptance criteria
- [ ] The ten gen-m6 cells below (case a4a301d00b46acf3) match Excel.
- [ ] `make eval-baseline` shrinks gen-m6; `make check` green; reviewers approve.

## Out of scope
- Other functions; the unrelated cells of these cases.

## Visible-parity stage
Decision 0083: every cell below is visibly off (the batch's 16 failures are all counted in the visible-parity lock). The acceptance criteria are the stage.
Cells:
- a4a301d00b46acf3 A79
- a4a301d00b46acf3 A82
- a4a301d00b46acf3 A85
- a4a301d00b46acf3 A88
- a4a301d00b46acf3 A241
- a4a301d00b46acf3 A244
- a4a301d00b46acf3 A307
- a4a301d00b46acf3 A310
- a4a301d00b46acf3 A493
- a4a301d00b46acf3 A499
