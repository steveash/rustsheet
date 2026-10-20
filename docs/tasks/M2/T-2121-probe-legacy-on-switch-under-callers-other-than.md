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
T-1964 found that a legacy cell puts no `@` on a SWITCH, its range or one-element results, or ABS/LEN/unary minus over it (probes/legacy-switch-at-shapes). Left unprobed and kept as before: `=INDIRECT(SWITCH(1,1,"Z30:Z32"))`, `=OFFSET(Z30,0,0,SWITCH(1,1,3))`, `=INDEX(Z30:Z32,SWITCH(1,1,{1;2}))`, a SWITCH past the first argument, `=SWITCH(1,1,Z30:Z32)+0`, `=ABS(SWITCH(1,1,Z30:Z32))`, `=IF(1,SWITCH(1,1,{2;3}))`, `=SWITCH(1,1,IF(1,{2;3}))`, and a range or array SWITCH lookup value or case. Code: `legacy_call_reads_a_value`, `is_switch_result` (static_analysis/legacy.rs). Use the excel-probe skill with legacy cells and FORMULATEXT twins like legacy-switch-at-shapes.

## Acceptance criteria
- [ ] A probes case covers the shapes above with oracle values; the parser matches them (baseline shrinks or the findings are recorded in references.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
