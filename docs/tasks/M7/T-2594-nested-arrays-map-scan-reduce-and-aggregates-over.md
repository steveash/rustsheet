---
id: T-2594
title: "Nested arrays: MAP, SCAN, REDUCE and aggregates over a nested GROUPBY/PIVOTBY cell (SPEC-0018)"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2592, T-2593]
---

## Goal
MAP, REDUCE and the aggregates read a nested GROUPBY/PIVOTBY cell as Excel does (SPEC-0018 rules 5 and 7, T-2592's evidence).

## Context
docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md rule 5 and T-2592's probe. Owned: probes/t2311-pivotby-value-columns-error P!D12
(`MAP(PIVOTBY(..,LAMBDA(v,v)),..)` is an error, ours a value) and the cells T-2592's probe adds that T-2593 leaves failing.
If T-2592 shows a reference LAMBDA body (rule 7) holds a nested array too, this task makes `Model::aggregate` keep it; if that grows the task past size S, split it (planner) first.

## Acceptance criteria
- [ ] P!D12 and T-2592's remaining cells pass and leave eval/baseline, with no new known-failure lines.
- [ ] arrays.md records each rule with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SCAN over a GROUPBY result (T-2398).
