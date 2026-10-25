---
id: T-2517
title: "Probe COMBINA just under the 2^31-1 cut for small number (COMBINA(1,2^31-2), COMBINA(2,2^31-3))"
milestone: M5
lane: research
status: in-progress
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
- [ ] Those cells are in a probes case, oracle-filled (excel-graph), and numbers.md COMBINA bound cites them; a fix task is filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing COMBIN or COMBINA (the fix task's).
