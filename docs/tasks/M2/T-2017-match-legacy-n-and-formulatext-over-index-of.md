---
id: T-2017
title: "Match legacy N and FORMULATEXT over INDEX of a scalar with a 0 index (T-1943)"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy `=N(INDEX(5,0,0))` and `=N(INDEX(5,0))` give `#VALUE!` and FORMULATEXT shows no `@`, as Excel does.

## Context
probes/index-zero-index-shape E10, E20, F10, F20: Excel reads INDEX of a scalar with an index of 0 as an array (like the 1x1 array forms; docs/excel-semantics/lookup.md "INDEX of an array: a whole row stays an array"), so the legacy N marker gives `#VALUE!` and FORMULATEXT omits `@`. The engine gives 5 and `=N(@INDEX(5,0,0))`. The dynamic TYPE/ROWS/COLUMNS cells already match.

## Acceptance criteria
- [ ] E10, E20, F10, F20 of probes/index-zero-index-shape pass; no other cell changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
