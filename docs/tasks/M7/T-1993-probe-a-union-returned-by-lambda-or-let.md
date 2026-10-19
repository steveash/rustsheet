---
id: T-1993
title: "Probe a union returned by LAMBDA or LET into OFFSET and COUNTIF"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
What Excel gives for `OFFSET(LAMBDA(x,x)((A1,B1)),1,0)` and `COUNTIF(LAMBDA(x,x)((A1,B1)),">0")` (and the LET forms) is pinned by oracle cells, and RustSheet matches it.

## Context
T-1951 review (excel-fidelity, minor): `evaluate_call_with_reference` collapses a union with `into_single_area` for every `evaluate_node_with_reference` caller; the error Excel gives there is unprobed.

## Acceptance criteria
- [ ] A probe case has excel-graph values and every cell passes; findings in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/lambda-union-offset-countif` (10 cells, excel-graph) shows Excel gives `#VALUE!` for a union passed to OFFSET or COUNTIF, direct or returned by LAMBDA/LET. Ours already matched (10/10); finding recorded in docs/excel-semantics/references.md. Eval delta: +10 passing cells, no baseline change.
