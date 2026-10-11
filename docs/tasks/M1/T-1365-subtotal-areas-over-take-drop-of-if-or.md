---
id: T-1365
title: "SUBTOTAL/AREAS over TAKE/DROP of IF or LET holding a computed array: spill like any array (follow-up of T-1332)"
milestone: M1
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`SUBTOTAL(9,TAKE(IF(TRUE,SEQUENCE(4)),2))` and the AREAS, DROP and LET forms spill two `#VALUE!` as Excel does (probes/subtotal-reference-operands-e: the spill cells I2, J2, O2, P2, R2:U2 are the known failures; the row-1 cells already give `#VALUE!`).

## Context
`is_read_reference` (static_analysis/aggregate.rs) treats TAKE/DROP/TRIMRANGE as a read reference when `is_reference_operand` accepts the first argument, and that accepts any IF/LET/CHOOSE/SWITCH/IFS call. When the call's branch or body is a computed array (not a reference) Excel spills like any array; the engine gives a scalar `#VALUE!`. Needs a check of what the IF branches / LET body evaluate to (a LET body may name a variable bound to either kind). Evidence: references.md, T-1332 entry. Excel gives references for `TAKE(IF(TRUE,A1:A4),2)` (SUBTOTAL 3) and `TAKE(LET(x,A1:A4,x),2)`, which must keep working.

## Acceptance criteria
- [ ] The eight failing cells of probes/subtotal-reference-operands-e pass; eval baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
