---
id: T-2232
title: "Probe CELL info_type grid elements that are errors or non-text (array and range)"
milestone: M9
lane: functions-logic
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CELL over an info_type grid (array or multi-cell range) has oracle evidence for error and non-text elements, and its spilled values.

## Context
`lifted_cell` (functions/information/mod.rs) turns every non-text element into `#VALUE!`; the scalar path passes an error through (`CELL(A1,B1)` with A1=#N/A is `#N/A`). Unprobed (T-2161 review): `CELL({#N/A,"row"},B2)`, `CELL(D1:D2,B2)` with D1=#DIV/0!, and the spilled values of a range info_type (`CELL(D1:D2,B3)`, D1="row", D2="col").

## Acceptance criteria
- [ ] A probe filled by excel-graph covers those cells; `lifted_cell` and information.md match it; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
