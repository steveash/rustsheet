---
id: T-2121
title: "Probe legacy @ on SWITCH under callers other than ABS/LEN, operators and nested in IF"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT and values are known for a legacy cell with a SWITCH under callers other than ABS and LEN, under binary operators, and nested in IF arms or SWITCH results, and ours matches.

## Context
T-1964 found that a legacy cell puts no `@` on a SWITCH at the root whose results are array constants or ranges (one-element included), nor on ABS, LEN or a unary minus over a SWITCH of array constants, nor on their results (probes/legacy-switch-at-shapes). The engine leaves only those shapes unmarked (`spilling_switch_position`, static_analysis/legacy.rs); everything else keeps the `@` it had before T-1964, unprobed: other callers (`=SIN(SWITCH(1,1,{2;3}))`, `=ROUND(..)`, `=N(..)`, a SUM-like vector argument `=SUM(SWITCH(1,1,Z30:Z32))`, `=INDEX(SWITCH(1,1,Z30:Z32),1)`, `=INDIRECT(SWITCH(1,1,"Z30:Z32"))`, `=OFFSET(Z30,0,0,SWITCH(1,1,3))`, `=INDEX(Z30:Z32,SWITCH(1,1,{1;2}))`), a SWITCH past the first argument, binary operators (`=SWITCH(1,1,{2;3})+0`, `=ABS(SWITCH(1,1,{-2;3}))+0`), parentheses (`=ABS((SWITCH(1,1,{-2;3})))`), a range result under ABS, LEN or minus (`=ABS(SWITCH(1,1,Z30:Z32))`), results that are operations or calls (`=SWITCH(1,1,Z30:Z32*1)`, `=SWITCH(1,1,ABS({-2;3}))`, `=SWITCH(1,1,IF(1,{2;3}))`, a nested SWITCH), a SWITCH in an IF arm (`=IF(1,SWITCH(1,1,{2;3}))`), and a range or array lookup value or case. Use the excel-probe skill with legacy cells and FORMULATEXT twins like legacy-switch-at-shapes.

## Acceptance criteria
- [ ] A probes case covers the shapes above with oracle values; the parser matches them (baseline shrinks or the findings are recorded in references.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
