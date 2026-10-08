---
id: T-1150
title: "Probe the remaining for_each_value aggregates with a union argument (HARMEAN, AVEDEV, SKEW, KURT, ...)"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for every aggregate that T-1062 put on the union-taking path without a probe of its own,
so docs/excel-semantics/references.md "Multi-area references in functions" lists none as unprobed.

## Context
T-1062 moved every user of `for_each_value` / `for_each_value_a` (functions/statistical/count_and_average.rs)
onto `Model::evaluate_reference_arg`, so they all walk a union's areas. probes/multiarea-agg, multiarea-rank and
multiarea-more verify SUM, COUNT, COUNTA, AVERAGE, AVERAGEA, MIN, MAX, MAXA, PRODUCT, MEDIAN, STDEV.S, SUMSQ,
GEOMEAN, VAR.S and DEVSQ. The rest are unprobed: HARMEAN, AVEDEV, SKEW, KURT, MINA, STDEV.P, STDEVA, VAR.P,
VARA, MODE.SNGL and the other callers (`rg -n "for_each_value" crates/rustsheet-engine/src`). PERCENTOF is
`#NAME?` in Excel for the web (probes/multiarea-more description), so it needs another backend (decision 0039).
Use the `excel-probe` skill; Sheet1 A1=1, B2=5, C3=7 as in the T-1061 cases.

## Acceptance criteria
- [ ] A probes case with one cell per listed function over a union, filled by excel-graph.
- [ ] references.md lists the results; any function Excel refuses gets an engine task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing engine behavior (file a task instead).
