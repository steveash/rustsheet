---
id: T-1589
title: "Oracle-confirm array lifting of CEILING, FLOOR, FLOOR.MATH/PRECISE, CEILING.PRECISE and MROUND (incl. boolean elements)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-evidence the array lifting T-1357 added for CEILING, FLOOR, CEILING.PRECISE, FLOOR.MATH, FLOOR.PRECISE and MROUND, and fix any divergence.

## Context
T-1357 lifted these by the general rule (arrays.md "CEILING/FLOOR family and MROUND lift over arrays"); only ISO.CEILING and CEILING.MATH have oracle cells. Open question: MROUND over a boolean array element (`MROUND({TRUE,2.6},1)`, a TRUE cell in a range, `MROUND(1,{TRUE,2})`); the engine gives `#VALUE!` per element (the scalar rule). Use the excel-probe skill. Code: functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] Probe rows for each function with an array in each argument, plus the MROUND boolean cases, with oracle values; engine matches; arrays.md status updated to VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
