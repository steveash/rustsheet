---
id: T-1206
title: "Probe SUBTOTAL/AREAS over arrays from TAKE/XLOOKUP, AREAS error elements, SUM of XLOOKUP cells (follow-up of T-1175)"
milestone: M1
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The UNVERIFIED rows T-1175 left in references.md "SUBTOTAL and AREAS take references only" are probed on excel-graph and the engine matches.

## Context
T-1175 sized XLOOKUP/TAKE/DROP/TRIMRANGE as scalars for SUBTOTAL/AREAS spill shape (`is_read_reference`, static_analysis/aggregate.rs). Unprobed: `SUBTOTAL(9,TAKE({1,2;3,4},1))` and `AREAS(TAKE({1,2},1))` (Excel may spill); `AREAS(IF(TRUE,{1,#N/A}))` (error element kept?); and whether SUM/AVERAGE/COUNT over a single-cell XLOOKUP/TAKE result holding text, a boolean or a blank treat it as a reference (`evaluate_picked_reference_arg` in functions/util.rs is limited to SUBTOTAL and AREAS). Extend probes/subtotal-reference-operands-b or add a case.

## Acceptance criteria
- [ ] A probe covers the cells above; references.md states each rule with the case as evidence.
- [ ] The engine matches; eval baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
