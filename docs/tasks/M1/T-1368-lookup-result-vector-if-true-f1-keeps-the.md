---
id: T-1368
title: "LOOKUP result_vector @IF(TRUE,F1) keeps the reference (engine gives #N/A)"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`=LOOKUP(99,A1:A3,@IF(TRUE,F1))` evaluates like `=LOOKUP(99,A1:A3,F1)`: `@` of an IF that picks one cell keeps the reference.

## Context
Probe `probes/at-if-spill-anchor-operand` Sheet1!H50: Excel gives 1, the engine #N/A (H60, the plain `LOOKUP(99,A1:A3,F1)`, already matches). A scalar result_vector (`LOOKUP(99,A1:A3,77)`, H70) is #N/A in Excel. Found by T-1337; same family as T-1321 (`spill_anchor_operand`).

## Acceptance criteria
- [ ] `probes/at-if-spill-anchor-operand` Sheet1!H50 passes; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
