---
id: T-2919
title: "Probe DROP as TAKE: handed-on lifted grid and error source"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DROP's lifted grid handed on by IF/CHOOSE/OFFSET, and DROP over an error source with a lifted count, are probed against Excel and match what is implemented (by analogy with TAKE, T-2299).

## Context
T-2299 added DROP to `may_hand_on_grid` and the error-source branch of `take_drop_lifted` with oracle evidence for TAKE only. Probe `ROWS(DROP(NA(),{1;2}))`, `SUMPRODUCT(IF(TRUE,DROP(A1:B3,{1;2})))`, `ROWS(OFFSET(DROP(A1:B3,{1;2}),1,0))` and the error-source-vs-invalid-count precedence (`TAKE(NA(),{1;"x"})`) with the excel-probe skill; fix the code and lookup.md where Excel differs.

## Acceptance criteria
- [ ] Probe cells recorded and passing, or implementation corrected; lookup.md no longer says "unprobed" for them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
