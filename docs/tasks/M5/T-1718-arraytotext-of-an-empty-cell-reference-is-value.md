---
id: T-1718
title: "ARRAYTOTEXT of an empty cell reference is #VALUE!"
milestone: M5
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ARRAYTOTEXT` of a reference to a single empty cell is `#VALUE!` in both formats, as in Excel.

## Context
probes/arraytotext-number-text A14, A15 (excel-graph, T-1356): `=ARRAYTOTEXT(B1,1)` and `=ARRAYTOTEXT(B1)` with B1 empty are `#VALUE!`; the engine gives "". Code: functions/text/arraytotext.rs. Probe first whether a range holding an empty cell, or an omitted argument, differs.

## Acceptance criteria
- [ ] A14 and A15 pass and leave the baseline; the range and omitted-argument cases are probed and recorded in docs/excel-semantics/text.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
