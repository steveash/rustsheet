---
id: T-1106
title: "LET and LAMBDA results that are references stay references (SUM(LET(x,K1,x)) is 0)"
milestone: M7
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A LET or LAMBDA whose result is its variable bound to a reference returns that reference, so aggregates skip text in it as Excel does.

## Context
probes/min-max-reference-forms Sheet1!D8 `=SUM(LET(x,K1,x))` with K1 = "text" is 0 in Excel (excel-graph,
2026-10-07) but `#VALUE!` here: since T-0223 a variable keeps its reference (`variable_value`, model.rs), but
`fn_let` evaluates its body with `evaluate_node_in_context`, where a single-cell variable becomes the cell's
value. Likely fix: evaluate the body with `evaluate_node_with_reference` when the caller wants a reference
(`evaluate_aggregate_arg`, `evaluate_node_with_reference` routing `Function::Let`/LAMBDA calls), keeping
`=LET(x,K1,x)` in a cell = "text". docs/excel-semantics/references.md, "Names, LET/LAMBDA variables and @
are references". Probe the LAMBDA form (`SUM(LAMBDA(x,x)(K1))`) and ISREF(LET(..)) first.

## Acceptance criteria
- [ ] probes/min-max-reference-forms D8 passes (baseline shrinks); new probe cells for the LAMBDA and ISREF forms match the oracle.
- [ ] 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
