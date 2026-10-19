---
id: T-1106
title: "LET and LAMBDA results that are references stay references (SUM(LET(x,K1,x)) is 0)"
milestone: M7
lane: functions-logic
status: done
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
- [x] probes/min-max-reference-forms D8 passes (baseline shrinks); new probe cells for the LAMBDA and ISREF forms match the oracle.
- [x] 0 eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`SUM(LET(x,K1,x))` (min-max-reference-forms D8) already passed on main. What remained:
`ISREF(LET(x,A1:A3,x))` was FALSE (written-parens-reference-only E40, E45) and a LAMBDA call
lost a one-cell reference (`SUM(LAMBDA(x,x)(K1))` was `#VALUE!`, `ISREF(LAMBDA(x,x)(A1))`
FALSE). New probe probes/let-lambda-reference-results (excel-graph, 18 cells) pins the LET and
LAMBDA forms. Now `is_reference` treats LET and a LAMBDA call like IF/CHOOSE (reference iff their
reference-context result is one), and `evaluate_reference_arg` reads a written LAMBDA call through
`call_callee_as_reference` (the caller's `keeping_references` mode). Recorded in references.md.
Eval delta: probes known failures 2096 -> 2094 (E40, E45 fixed), +1 case (18 cells, all pass),
0 regressions.
