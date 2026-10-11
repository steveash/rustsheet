---
id: T-1276
title: "Spec: GROUPBY and PIVOTBY"
milestone: M7
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ for GROUPBY and PIVOTBY (grouping, aggregation through a LAMBDA or eta name, headers, totals, sorting, filtering, field_relationship, relative_to) with oracle evidence, cut into implementable tasks.

## Context
Neither function exists in the engine (`#NAME?`). T-1083's probe probes/m7-argument-modes rows 33-35 (docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"): `GROUPBY(D1:D3,A1:A3,SUM)` is 3x2 (two groups and a total row), PIVOTBY 4x4 with a blank corner. The generator calls both (spelled `_xlfn.GROUPBY`: the case builder does not prefix them, since probes/catalog/function-name-forms needs them bare). This is size L as an implementation, so it needs a spec first (CLAUDE.md directive 4); use the excel-probe skill for every argument's behaviour.

## Acceptance criteria
- [ ] docs/specs/ has the spec with probe evidence; implementation tasks are filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
