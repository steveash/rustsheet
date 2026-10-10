---
id: T-1195
title: "Cache the contains-SUBTOTAL/AGGREGATE flag per parsed formula"
milestone: M1
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL and AGGREGATE decide once per parsed formula, not once per visited cell, whether it calls SUBTOTAL or AGGREGATE.

## Context
T-1183 made `cell_formula_is_one_of` (functions/subtotal.rs) walk the whole tree with `Node::contains_function` for every visited formula cell. `parsed_formulas[sheet][f]` is shared, so a column filled from one formula is walked once per cell, each walk allocating a stack Vec. Constant factor only (performance review of T-1183). Store the flag beside the parsed formula or cache it lazily per formula index.

## Acceptance criteria
- [ ] A SUBTOTAL over a column of one shared formula walks the tree once; probes/subtotal-hidden-filtered still passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
