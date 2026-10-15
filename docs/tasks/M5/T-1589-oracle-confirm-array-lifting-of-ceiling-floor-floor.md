---
id: T-1589
title: "Oracle-confirm array lifting of CEILING, FLOOR, FLOOR.MATH/PRECISE, CEILING.PRECISE and MROUND (incl. boolean elements)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-evidence the array lifting T-1357 added for CEILING, FLOOR, CEILING.PRECISE, FLOOR.MATH, FLOOR.PRECISE and MROUND, and fix any divergence.

## Context
T-1357 lifted these by the general rule (arrays.md "CEILING/FLOOR family and MROUND lift over arrays"); only ISO.CEILING and CEILING.MATH have oracle cells. Open question: MROUND over a boolean array element (`MROUND({TRUE,2.6},1)`, a TRUE cell in a range, `MROUND(1,{TRUE,2})`); the engine gives `#VALUE!` per element (the scalar rule). Also probe: `MROUND(1/0,{TRUE,2})` and `MROUND(1/0,TRUE)` (error vs boolean precedence), `MROUND(INDIRECT("A1"),1)` with A1=TRUE, and write the scalar MROUND quirk (missing arg `#N/A` before boolean `#VALUE!`) into docs/excel-semantics. Use the excel-probe skill. Code: functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] Probe rows for each function with an array in each argument, plus the MROUND boolean cases, with oracle values; engine matches; arrays.md status updated to VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New case `probes/ceiling-floor-mround-array-lift` (65 cells, excel-graph): every function with an array in each argument, boolean elements, and MROUND error/missing/boolean precedence. The lift and the boolean-element rules already matched; MROUND's argument checks were wrong: Excel judges them left to right (`MROUND(1/0,TRUE)` is `#DIV/0!`, `MROUND(TRUE,)` is `#VALUE!`), the engine checked every missing argument before any boolean or error. Fixed in `mathematical.rs`; arrays.md is VERIFIED. Eval delta: 0 fixed, 1 new case passing 65/65, 0 regressions.
