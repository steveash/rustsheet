---
id: T-1205
title: "Probe unnamed multi-cell operands of the range operator and undefined column-spelled names before a colon"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for `:` with unnamed multi-cell operands and for an undefined column-spelled identifier before a colon, with the engine matching.

## Context
T-1173 made `:` join any same-sheet operands into a bounding box and made the lexer read a column-spelled identifier before a non-column-range `:` as a name. Evidence covers only named operands (probes/defined-name-grouped-reference A10, A11). Probe `=ROWS(OFFSET(B1,0,0,2,1):B3)`, `=SUM(INDEX(B1:B3,1):INDEX(B1:B3,3))`, and `=SUM(A:B3)`, `=SUM(xx:B3)`, `=ig:B3` with `ig` undefined (value, and whether Excel accepts the formula).

## Acceptance criteria
- [ ] Probe cells with excel-graph sidecar; references.md updated; engine matches or mismatches baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Corner order and cross-sheet operands.
