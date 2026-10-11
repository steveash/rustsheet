---
id: T-1332
title: "Probe SUBTOTAL/AREAS over XLOOKUP and TAKE with computed-array sources (IF, LET, names) (follow-up of T-1211)"
milestone: M1
lane: functions-math
status: todo
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
- [ ] A probe covers these forms; references.md states each rule with the case as evidence; the engine matches or a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
