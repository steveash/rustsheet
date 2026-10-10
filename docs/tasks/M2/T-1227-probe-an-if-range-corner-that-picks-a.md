---
id: T-1227
title: "Probe an IF range corner that picks a union, an error or uses an array condition"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the IF range corners T-1177 left unprobed, with the engine matching it.

## Context
T-1177 (probes/isref-range-operands) showed that Excel treats an IF corner of `:` as the
reference it picks (`SUM(A1:IF(TRUE,B2))` = 33). `range_operand` in
crates/rustsheet-engine/src/reference_operators.rs now evaluates IF through
`evaluate_reference_arg(..).into_single_area(..)`. These cases are engine choices with no oracle
evidence yet:
- IF picks a union: `SUM(A1:IF(TRUE,(B1,B2)))`, `ISREF(A1:IF(TRUE,(B1,B2)))` (engine: `#VALUE!`, FALSE).
- IF picks an error: `ERROR.TYPE(A1:IF(TRUE,#N/A))`, `ERROR.TYPE(A1:IF(TRUE,INDEX(B1:B3,9)))`.
- IF with an array condition: `SUM(A1:IF({TRUE,FALSE},B2,B3))`, `ISREF(A1:IF({TRUE},B2))`.
- IF picks a 3-D span: `SUM(A1:IF(TRUE,Data1:Data3!B2))` (engine: `#VALUE!`).
See docs/excel-semantics/references.md, "Sheet-qualified range corners: order and unprobed forms".

## Acceptance criteria
- [ ] A probe case with the cells above, oracle-filled (excel-graph, excel-probe skill).
- [ ] The engine matches, or each mismatch has its own engine task; references.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CHOOSE corners (already verified in probes/isref-range-operands D8, D9, E8, E9).
