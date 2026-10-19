---
id: T-1951
title: "Probe LAMBDA calls returning a union or a one-cell reference to OFFSET, COUNTIF and AREAS"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1106]
---

## Goal
What Excel does with a written LAMBDA call whose body returns a union, or a one-cell reference passed on to OFFSET, COUNTIF or AREAS, is pinned by oracle cells, and RustSheet matches it.

## Context
T-1106 review (excel-fidelity, minor). `call_callee_as_reference` (functions/logical/lambda.rs)
reads a LAMBDA call's body in reference context but collapses it with `into_single_area`, so
`SUM(LAMBDA(x,x)((A1,B1)))` and `ISREF(LAMBDA(x,x)((A1,B1)))` lose the union, while
`SUM(LET(x,(A1,B1),x))` keeps it (`let_impl(.., true)`). A one-cell result now reaches every caller of
`evaluate_reference_arg` as a reference (OFFSET, COUNTIF ranges, INDEX), which only ROWS, SUM, MAX,
COUNTA and ISREF pin (probes/let-lambda-reference-results). Probe with the excel-probe skill:
`SUM`/`ISREF`/`AREAS` of `LAMBDA(x,x)((A1,B1))`, `OFFSET(LAMBDA(x,x)(A1),1,0)`,
`COUNTIF(LAMBDA(x,x)(A1:A3),">1")`, then implement what the oracle shows (return the `RefArg`
uncollapsed if Excel keeps the areas).

## Acceptance criteria
- [x] A probe case with the forms above has excel-graph values; every cell passes (or failures are filed).
- [x] The findings are recorded in docs/excel-semantics/references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- LAMBDA helpers (MAP, REDUCE, BYROW) returning references.

## Outcome
Probe probes/lambda-union-reference-callers (excel-graph): Excel keeps a union and a one-cell reference through a LAMBDA call
into SUM, ISREF, AREAS, INDEX, OFFSET and COUNTIF. Fixed by returning the `RefArg` uncollapsed from
`call_callee_as_reference` and reading LET/LAMBDA calls as references in `evaluate_node_with_reference`. Findings in
docs/excel-semantics/references.md.
