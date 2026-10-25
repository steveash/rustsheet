---
id: T-2517
title: "Probe COMBINA just under the 2^31-1 cut for small number (COMBINA(1,2^31-2), COMBINA(2,2^31-3))"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's COMBINA just under the n+k-1 cut is known from oracle cells for a small `number`, so the T-2291 probe is bounded from both sides.

## Context
T-2291 probed only cells at or past the cut for small `n`. Probe `COMBINA(1,2^31-2)` (expect 1) and `COMBINA(2,2^31-3)` (expect 2147483646) with the `excel-probe` skill; a cut on k alone a little lower would also fit T-2291's data.

## Acceptance criteria
- [x] Those cells are in a probes case, oracle-filled (excel-graph), and numbers.md COMBINA bound cites them; a fix task is filed if ours differs.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing COMBIN or COMBINA (the fix task's).

## Outcome
Added probes/t2517-combina-below-cut-small-n (excel-graph filled): `COMBINA(1,2^31-2)` = 1, `COMBINA(2,2^31-3)` = 2147483646,
`COMBINA(1,2^31-3)` = 1, `COMBINA(3,2^31-4)` = 2.3058430038449848E+18, all numbers. numbers.md COMBINA bound cites them.
Ours matches Excel on all four cells (the eval baseline lists no failing cell for the case), so no fix task is needed.
Eval delta: one new probe case, 4 cells, 0 known failures.
