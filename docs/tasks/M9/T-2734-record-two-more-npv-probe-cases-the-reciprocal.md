---
id: T-2734
title: "Record two more NPV probe cases: the reciprocal-overflow boundary near -1, error cells in references, and gen-m9 A255's inner calls"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The NPV probes written for a duplicate T-2493 attempt (#1819, closed) are kept as evidence for the rules #1818 landed.

## Context
probes/t2493-npv-rate-edges (A1:A77) and probes/t2493-npv-error-order-inner-calls (excel-graph, 2026-10-25) cover cases
main's probes/npv-power-edges and -2 do not:
- the boundary near -1, where a typed -0.9999999999999999 is cut to 15 digits: 20 flows give a value, 21 give #NUM!,
  22 or more give #DIV/0!;
- an error flow before a failing discount (`NPV(-1,NA())` is #N/A);
- error cells in a referenced range against a scalar error;
- the inner calls of gen-m9/0a957119475760c5 A157 and A255.
Main's engine passes all of them except A60 and A61, which are 1 ulp off (NPV last digits, handed to T-1528).

## Acceptance criteria
- [x] Both cases are in eval/suites/probes with excel-graph sidecars; `make eval-baseline` records them; financial.md cites them.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (#1818 has them).

## Outcome
Done. Both cases are recorded with their excel-graph sidecars, and financial.md "NPV's error order" and the discount
section cite them. Main's engine (#1818) passes every cell except A60 and A61, which are 1 ulp off and handed to T-1528.
The rate-edges case description still names A15:A25, which are in the inner-calls case; it is left as filled, because
the sidecar records the case's input.
