---
id: T-1466
title: "Eta-reduced function names (_xleta.) as LAMBDA arguments; BYCOL(range,COUNTA) is #VALUE!"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A function passed by name (stored `_xleta.NAME`) works wherever a LAMBDA does, and `BYCOL(A1:B3,COUNTA)` is `#VALUE!` as in Excel.

## Context
probes/lambda-helper-range-slices E32 (`BYCOL(A1:B3,COUNTA)`: Excel `#VALUE!`, engine `#NAME?`) and the eta-reduced cases of probes/m7-argument-modes. See docs/excel-semantics/arrays.md "A function passed by name" and T-1442's Outcome; the engine has no `_xleta.` handling.

## Acceptance criteria
- [ ] probes/lambda-helper-range-slices E32 passes and the m7-argument-modes eta cases that Excel evidences pass; `make eval-baseline` records the fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ranges on several sheets (3-D) and unions.
