---
id: T-2063
title: "Aggregates read an array of references: SUM-family top-left cells, COUNT/COUNTA/SUMPRODUCT rules (SPEC-0014)"
milestone: M6
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1956]
---

## Goal
SUM, AVERAGE, MIN, MAX and PRODUCT read each element of an array of references as its top-left cell (a range cell), and COUNT, COUNTA and SUMPRODUCT read the elements as non-numbers, as Excel does.

## Context
Design: docs/specs/SPEC-0014-reference-array-results.md rules 4 and 5, "Consumers" (the aggregate walker's `TopLeft`/`NotANumber` descriptor). Builds on T-1956's `RefGrid`. Evidence: probes/reference-lift-consumers E1-E29, E117, E125, E129, E133, E181-E201 and reference-lift-consumers-b2 E117-E121, E145, E157-E169, E181-E189, E201, E213 (excel-graph, 2026-10-19). Code: SPEC-0009's walker (functions/util.rs `evaluate_reference_arg`, functions/aggregate.rs, functions/statistical/count.rs).

## Acceptance criteria
- [ ] The cells above pass and leave eval/baseline (`make eval-baseline`), with no new known failures.
- [ ] Aggregates not named in SPEC-0014 rules 4-5 keep the value view (unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
