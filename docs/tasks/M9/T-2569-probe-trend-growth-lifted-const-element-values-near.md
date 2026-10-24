---
id: T-2569
title: "Probe TREND/GROWTH: lifted const element values, near-collinear x, a 0 GROWTH y against a 2x2 x"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles three TREND/GROWTH choices T-1726 made without it.

## Context
T-1726 (functions/statistical/trend.rs; statistics.md "TREND: defaults, shapes, single points and error inputs") left
three rules unprobed:
- the values of a lifted array const's elements: ours takes each element's top-left value
  (`TREND(A1:A6,B1:B6,K1,{TRUE,FALSE})` is {7.2, 8.307692308}; test_fn_trend.rs `an_array_const_lifts`), while UNIQUE and
  SORTBY leave later elements empty (arrays.md "UNIQUE's and SORTBY's lifted elements");
- the singular-fit fallback `independent_columns` (Gram-Schmidt, relative cut-off 1e-10) for near-collinear x columns
  (for example E = 2*A + 1E-12) and an all-zero x column without a constant term;
- gen-m9 abc2939ebd1e2365 A187: `GROWTH(PHI(-1E+300),Data!C11:D12,Data!E5:E9,Data!D1)`, a 0 y against a 2x2 x, is
  `#NUM!` in Excel (ours the size `#REF!`); probe a non-positive single y against wrongly sized x for TREND and GROWTH,
  and `TREND(5,2,4,FALSE)` (one point without a constant term), plus GROWTH predictions that overflow `exp`.

Also (T-1741): probe LINEST's collinear column choice with y not in x (`LINEST(y,A:C)` with C = 2A): ours drops
A, keeping the last column first (from linest-modes K106 alone); `test_linest_drops_a_collinear_column` pins that
and must follow the probe.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with oracle values for each bullet under Context is committed under eval/suites/probes.
- [ ] statistics.md records the findings; the UNVERIFIED clauses of the TREND entry are resolved or kept with the task id.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (file a functions-stat task for any mismatch).
