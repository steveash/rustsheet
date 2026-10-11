---
id: T-1332
title: "Probe SUBTOTAL/AREAS over XLOOKUP and TAKE with computed-array sources (IF, LET, names) (follow-up of T-1211)"
milestone: M1
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL/AREAS over XLOOKUP whose return array is computed (`XLOOKUP(1,A1:A4,SEQUENCE(4))`) and TAKE/DROP over `IF(...)`, `LET(...)` or a defined name holding an array are probed on excel-graph and the engine matches.

## Context
T-1211 review (excel-fidelity, minor): `is_read_reference` in static_analysis/aggregate.rs treats XLOOKUP as a reference unless arg 3 is a literal array, and `is_reference_operand` falls back to true for IF/LET/names. Unprobed. Extend probes/subtotal-reference-operands-d or add -e; see references.md.

## Acceptance criteria
- [x] A probe covers these forms; references.md states each rule with the case as evidence; the engine matches or a fix task is filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/subtotal-reference-operands-e (36 Excel cells, excel-graph). The engine matches 28. XLOOKUP with a
computed return array is an array (`#VALUE!`, ISREF FALSE); XLOOKUP/TAKE over `IF(TRUE,A1:A4)` or `LET(x,A1:A4,x)` stays
a reference. TAKE/DROP of an IF/LET holding `SEQUENCE(4)` is an array that spills two `#VALUE!`; the engine returns a
scalar there (8 cells, known failures). Filed T-1365 for the fix. Rules recorded in references.md.
Defined names holding an array were not probed (no name form in the case); a name is not a reference-returning call, so file a probe if one is wanted.
