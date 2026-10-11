---
id: T-1276
title: "Spec: GROUPBY and PIVOTBY"
milestone: M7
lane: research
status: done
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
- [x] docs/specs/ has the spec with probe evidence; implementation tasks are filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
docs/specs/SPEC-0006-groupby-pivotby.md specifies both functions from a new probe, probes/groupby-pivotby: 61 formulas covering every argument, excel-graph, all spill cells recorded. The rules are in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY". The rollout is filed as T-1322..T-1328: the GROUPBY core, options, subtotals, function arrays/eta, PIVOTBY, relative_to, and a probe follow-up for the four surprising or undecoded answers. Eval delta: +1 case, 1432 new-coverage known failures (nothing implemented yet); no regressions.
