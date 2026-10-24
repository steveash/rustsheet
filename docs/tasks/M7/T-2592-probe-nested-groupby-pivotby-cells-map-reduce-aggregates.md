---
id: T-2592
title: "Probe nested GROUPBY/PIVOTBY cells: MAP, REDUCE, aggregates and text over them, reference LAMBDAs, two levels (SPEC-0018)"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-graph evidence for every unknown SPEC-0018 lists about GROUPBY/PIVOTBY cells that hold a LAMBDA's array result, recorded in arrays.md.

## Context
docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md "Excel behavior to match" lists the unknowns. Over the D sheet of probes/t2360-array-lambda-holidays
(`GROUPBY(D!A1:A5,D!C1:D5,LAMBDA(v,v))` and the PIVOTBY beside it), probe, reading errors through ERROR.TYPE (excel-graph
returns rich errors it cannot decode):
- the error MAP, BYROW and BYCOL give over the result (t2311-pivotby-value-columns-error P!D12 shows only that it is one);
- SUM, COUNT, COUNTA, AVERAGE, ARRAYTOTEXT, TEXTJOIN, CONCAT and `&` over the whole result;
- REDUCE over it (SCAN is T-2398's);
- `+0`, `--` and `&""` on one INDEXed nested cell (its top-left or the array lifted);
- `A1#` over a spilled result (each nested cell read as `#VALUE!`?);
- TYPE via INDEX for `LAMBDA(x,D!D2)` (a reference body) and a two-parameter LAMBDA (rule 7: nested or the error);
- two levels: a LAMBDA whose result is itself a GROUPBY with `LAMBDA(v,v)`.
- MAP with a scalar step (`MAP(..,LAMBDA(x,ROWS(x)))`, `TYPE(x)`): does MAP refuse a nested element or hand it the array?
- shape functions over the result (TAKE, CHOOSECOLS, VSTACK, SORT, FILTER) and an INDEX selecting a whole row or column:
  do they keep nested elements?
- a subtotal cell (two row fields) and a computed 1x1 result (`LAMBDA(v,{7})`, `SEQUENCE(1)`): nested or unwrapped?
Use the excel-probe skill; one case file.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values for each question above.
- [ ] arrays.md (the "A LAMBDA array result is shown as `#VALUE!`" paragraph) records every answer; SPEC-0018's unknowns
      list points at it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-2593, T-2594).
