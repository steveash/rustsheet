---
id: T-1758
title: "A call of a cell-shaped name is a call of that cell"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`=Ab1(1)` is `#REF!` and FORMULATEXT prints `=AB1(1)`, as in Excel: a callee spelled like an A1 cell reference is a call of that cell, not of an unknown name.

## Context
Found by T-1719: probes/unknown-function-name-spelling A8 (`#REF!`, ours `#NAME?`) and B8 (`=AB1(1)`, ours `=Ab1(1)`); rule in docs/excel-semantics/references.md "Unknown function call names". `parse_named_call` (parser/operands.rs) makes any name that is no function a `NamedFunctionKind`; a cell-shaped one should become a `LambdaCallKind` of the reference (the evaluator already makes a called reference `#REF!`, `call_callee`). Known functions shaped like cells (`LOG10`, `ATAN2`, `DAYS360`) must stay functions: only a name no function lookup knows. Probe first which shapes count (a column past XFD, `$`-anchored, a row past 1048576, R1C1-shaped names like `R1C1(1)` in A1 mode) if the evidence does not cover them.

## Acceptance criteria
- [ ] probes/unknown-function-name-spelling A8 and B8 pass; any new probe cells pass; the rule in docs/excel-semantics/ is updated.
- [ ] `=LOG10(100)` and other cell-shaped function names are unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`parse_named_call` turns a callee spelled like an in-range A1 cell, and no function, into a `LambdaCallKind` of that cell (`#REF!`); `callee_operand` prints a bare cell callee unparenthesized (`AB1(1)`). Probe cells unknown-function-name-spelling A8/B8 now pass; unit test added. Edge shapes (past XFD, `$`, R1C1) remain unprobed and are noted in references.md.
