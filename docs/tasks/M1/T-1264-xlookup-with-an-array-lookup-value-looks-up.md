---
id: T-1264
title: "XLOOKUP with an array lookup_value looks up each element"
milestone: M1
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XLOOKUP with an array `lookup_value` (`SUM(XLOOKUP({1;2},A1:A3,B1:B3))`) returns one result per element, as Excel does.

## Context
probes/let-name-uncalled-picked-refs Sheet1!H21: Excel gives 30, the engine `#N/A` (`xlookup_impl`, functions/xlookup.rs, evaluates lookup_value to a single value). Probe the shapes first (row/column/2-D lookup_value, with if_not_found and the reference form) with the excel-probe skill, and record them in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [ ] H21 passes and its baseline entry is removed; probed shapes are recorded in docs/excel-semantics/lookup.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
