---
id: T-1321
title: "ANCHORARRAY(@IF(TRUE,D1)) spills the anchor's array (engine #VALUE!; T-1285)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ANCHORARRAY(@IF(TRUE,D1))` spills D1's array like Excel.

## Context
Excel spills 1,2,3 (D1 = SEQUENCE(3)) and prints `=@IF(TRUE,D1)#`; the engine gives #VALUE!. Failing cells probes/at-signed-operand.case.json Sheet1!H71:H73 (T-1285).

## Acceptance criteria
- [ ] `probes/at-signed-operand.case.json` H71:H73 pass and leave the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
