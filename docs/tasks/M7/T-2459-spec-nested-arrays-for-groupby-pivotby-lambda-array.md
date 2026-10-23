---
id: T-2459
title: "Spec: nested arrays for GROUPBY/PIVOTBY LAMBDA array results (TYPE 64 cells, top-left when lifted)"
milestone: M7
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ for how the engine represents a cell of a GROUPBY or PIVOTBY result that holds an array (a LAMBDA
array result), so INDEX returns that array, TYPE is 64, a consumer lifting over the result reads its top-left, and MAP over
it is an error, while the cell still displays `#VALUE!`; and implementation tasks cut from it.

## Context
Split from T-2360 (probes/t2360-array-lambda-holidays P, excel-graph, 2026-10-23; arrays.md, the paragraph starting
"A LAMBDA array result is shown as `#VALUE!`, but the cell holds the array itself"). Excel keeps the array per cell (even 1x1);
ours makes each such cell `#VALUE!`. The engine has no nested-array value (CalcResult::Array holds ArrayNode scalars), so this
is architectural (CLAUDE.md directive 4). Cells it owns: probes/t2311-pivotby-value-columns-error P!C12, P!D12; the 53 P!
known failures of probes/t2360-array-lambda-holidays (P!B3.. P!E37, C1, C9, C17, C25, C33); gen-m7/21b4eef18424ce94 A1504's 26
cells (T-2360's Context lists them). Open questions for the spec: what other functions keep nested arrays (BYROW/MAP give
`#CALC!` instead, arrays.md), what ARRAYTOTEXT, TEXTJOIN, COUNT and arithmetic do with such a cell, and how a spill stores it.

## Acceptance criteria
- [ ] docs/specs/ has the spec, with the probe evidence above and any new probe it needs.
- [ ] Implementation tasks (size S/M) are filed from it, owning the cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing it (the tasks the spec cuts).
