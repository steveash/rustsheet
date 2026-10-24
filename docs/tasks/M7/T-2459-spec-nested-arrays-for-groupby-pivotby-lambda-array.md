---
id: T-2459
title: "Spec: nested arrays for GROUPBY/PIVOTBY LAMBDA array results (TYPE 64 cells, top-left when lifted)"
milestone: M7
lane: research
status: in-progress
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
`#CALC!` instead, arrays.md), what ARRAYTOTEXT, TEXTJOIN, COUNT and arithmetic do with such a cell, how a spill stores it, and whether a LAMBDA whose body is a one-cell reference (`LAMBDA(x,Data!D2)`, also shown as `#VALUE!`) holds a nested array too (unprobed).
Also owned (from T-2398, probes/scan-over-groupby, excel-graph, 2026-10-23): `INDEX(GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,w)),4,2)`
spills the Total row's {1;2;3;4}, and SCAN MIN over that result is `#N/A` (scan-over-groupby A36, probes/groupby-lifted-args
A153/I153, gen-m7/b24ce74f08d3f91a A1850; ours `#VALUE!`). The spec covers what SCAN and REDUCE do with such a cell as well as
INDEX, TYPE and MAP, and bounds the memory a nested array may take (CLAUDE.md directive 5). T-2398 (draft #1583) depends on the
implementation task that makes those cells pass.

## Acceptance criteria
- [ ] docs/specs/ has the spec, with the probe evidence above and any new probe it needs.
- [ ] Implementation tasks (size S/M) are filed from it, owning the cells above, T-2398's SCAN cells included; T-2398 depends on the one that makes them pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing it (the tasks the spec cuts).
