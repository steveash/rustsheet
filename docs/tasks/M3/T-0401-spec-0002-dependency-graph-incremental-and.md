---
id: T-0401
title: "SPEC-0002: dependency graph, incremental and lazy recalculation"
milestone: M3
lane: engine
status: blocked
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Design the persistent dependency graph and calc scheduler (Automatic/Manual/OnDemand), volatile and dynamic dependencies, circular references, and how the current recursive evaluator is reused.

## Context
Arrays of references (SPEC-0014 "Dependency graph"): a lifted TAKE/DROP/TRIMRANGE grid reads cells inside its argument's static range, so it needs no dynamic dependency; an OFFSET or INDIRECT grid depends through volatility. docs/research/research-plan WS5; docs/research/api-landscape.md §4.5; Formualizer/HyperFormula designs. Multi-area references: SPEC-0004 "Dependency graph" lists what a union must register.

WEEKNUM readers (return_type 22+ or negative) depend on every earlier WEEKNUM/ISOWEEKNUM call in
the calc order, an edge the dependency graph does not have. SPEC-0015 "Incremental recalculation"
sets the rule SPEC-0002 must keep (T-1484).

## Acceptance criteria
- [ ] docs/specs/SPEC-0002-dependency-graph.md accepted by architecture+performance reviewers.
- [ ] Implementation tasks (S/M each) created by the planner with dependencies.
- [ ] `make check` green; reviewers approve.

## Out of scope
Implementation.

## Blocked because
Deferred to Phase 2 (decision 0027): the dependency-graph, storage and performance work waits until
Phase 1 parity exits. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`.
