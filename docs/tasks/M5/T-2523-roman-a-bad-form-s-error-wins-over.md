---
id: T-2523
title: "ROMAN: a bad form's error wins over a negative number (ROMAN(-5,NA()) is #N/A)"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ROMAN(-5,NA())` is `#N/A` as in Excel (ours `#VALUE!`).

## Context
Probe probes/roman-lift-forms Calc!G1 (excel-graph, T-2423); see numbers.md ROMAN section. The form's error must be checked before the negative-number `#VALUE!`; `fn_roman` in the engine.

## Acceptance criteria
- [ ] probes/roman-lift-forms passes in full; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
