---
id: T-2523
title: "ROMAN: a bad form's error wins over a negative number (ROMAN(-5,NA()) is #N/A)"
milestone: M5
lane: functions-math
status: done
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
- [x] probes/roman-lift-forms passes in full; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_roman` now converts the form (propagating its error) before the number's range check, so `ROMAN(-5,NA())` is `#N/A`
while `ROMAN(-5,"x")` stays `#VALUE!`. probes/roman-lift-forms now passes in full.
