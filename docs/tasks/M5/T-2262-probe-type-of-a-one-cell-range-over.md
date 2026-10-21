---
id: T-2262
title: "Probe TYPE of a one-cell range over text, boolean, error, blank, OFFSET and INDIRECT cells"
milestone: M5
lane: functions-logic
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TYPE(X:X)` over a one-cell range holding text, a boolean, an error, a blank, long text or a spill anchor, and over a one-cell OFFSET or INDIRECT result, is the answer Excel gives, recorded and matched.

## Context
T-2156 made `fn_type` (functions/information/mod.rs) classify the cell of a one-cell range; only numeric cells (gen-m5 A371, A203) and `TYPE(INDEX(Z1:Z2,1))` (lookup.md) have oracle evidence. The other cell kinds and the OFFSET/INDIRECT paths follow the same branch but are unprobed.

## Acceptance criteria
- [ ] One probe case covers `TYPE(A1:A1)` over blank, text, boolean, error, 256-unit text and spill-anchor cells, and one-cell OFFSET and INDIRECT; answers recorded in information.md and matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
