---
id: T-1175
title: "Probe SUBTOTAL/AREAS UNVERIFIED rows: two non-reference arguments, AREAS of an array, invalid function number (follow-up of T-1163)"
milestone: M1
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The three UNVERIFIED rows of references.md "SUBTOTAL and AREAS take references only" are probed on excel-graph and the engine matches.

## Context
T-1163 left these engine choices unprobed: which of two non-reference arguments wins (`SUBTOTAL(9,IF(TRUE,"a"),IF(TRUE,1/0))`; the engine takes the first), AREAS of an array (`AREAS(IF(TRUE,{1,2}))`; the engine gives one `#VALUE!`), and an invalid function number next to a non-reference argument (`SUBTOTAL(12,IF(TRUE,{1,2}))`, `SUBTOTAL(99,#REF!)`; the engine checks the function number first, `Model::subtotal_reference_args` in functions/subtotal.rs). Also probe that SUBTOTAL accepts OFFSET, XLOOKUP, IFS, SWITCH, TAKE, DROP, TRIMRANGE, INDIRECT and a defined name (one holding a constant should be `#VALUE!`); these come from the intersection rule only. Use only forms that load (see the refused list there); extend probes/subtotal-reference-operands or add a new case.

## Acceptance criteria
- [ ] A probe case covers the three rows and the extra operands; references.md states the rule with the case as evidence.
- [ ] The engine matches; eval baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
