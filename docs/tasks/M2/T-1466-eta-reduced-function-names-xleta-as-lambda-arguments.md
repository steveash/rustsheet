---
id: T-1466
title: "Eta-reduced function names (_xleta.) as LAMBDA arguments; BYCOL(range,COUNTA) is #VALUE!"
milestone: M2
lane: functions-logic
status: done
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
- [x] probes/lambda-helper-range-slices E32 passes and the m7-argument-modes eta cases that Excel evidences pass; `make eval-baseline` records the fixes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Ranges on several sheets (3-D) and unions.

## Outcome
An unbound `_xleta.NAME` of a known function (any case, `_xlfn.` allowed after the prefix) evaluates to a LAMBDA whose
definition is made per call with as many parameters as it is given (`Model::unbound_name`, `eta_definition`, lambda.rs;
`lambda_definition` now holds the shared call prologue). BYROW and BYCOL take only a LAMBDA and are `#VALUE!` for an
error, a number or an unknown name (`by_slices_lambda`), which fixes the bare `BYCOL(A1:B3,COUNTA)` of E32. New probe
probes/eta-reduced-names (30 questions, excel-graph 2026-10-14) pins the arity, case, prefix and refusal rules;
docs/excel-semantics/arrays.md "Functions passed by name (`_xleta.`)". Also closes T-1273 (superseded).
Eval delta: probes known failures 5142 -> 5132 (13 fixed: m7-argument-modes F28:H28, F48:H51, lambda-helper-range-slices
E32; 3 new-coverage failures in the new probe, E49, E93, E101), 0 regressions. gen-m7 (not gated): 7 `_xleta.` cells
of T-1273 now pass; gen-m9 (not gated): 6. Follow-up T-1511 (BYROW `#CALC!` for an array row, `ROWS(LAMBDA(x,x))`,
`ISERROR` of a LAMBDA, MAP over single values).
