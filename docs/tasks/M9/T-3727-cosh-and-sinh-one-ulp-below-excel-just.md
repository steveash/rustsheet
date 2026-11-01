---
id: T-3727
title: "COSH and SINH one ulp below Excel just above 709.7 (t3616 N8, N16, N21, O8, O16, O21)"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: []
---

## Goal
COSH and SINH give Excel's double just above x = 709.7, where Excel is one ulp above ours, without losing any other
COSH or SINH cell.

## Context
probes-numeric/t3616-imsin-imcos-positive-cut-probe (excel-graph, T-3616) has COSH (column N) and SINH (column O) of
22 values of x from 709.699999999999 to 709.75. 16 of each match; 3 of each are one ulp below Excel:
- N8/O8 `COSH(709.72)`, `SINH(709.72)`: Excel 8.442084613811263E+307, ours 8.442084613811262E+307.
- N16/O16 x = 709.7000001: Excel 8.274920965893093E+307, ours ...092E+307.
- N21/O21 x = 709.700000000001: Excel 8.274920138408849E+307, ours ...848E+307.
At this x, e^-x is negligible, so both are `e^x/2` (numbers.md "SINH / COSH / TANH are exp-based, not libm"). The
miss is then EXP's last bit or the halving order (`exp(x)/2` against `exp(x - ln 2)`); EXP itself (column M) matches
at all 22 points. Related: T-2744 (two COSH points of the unit sweep, also one ulp).
All 6 cells are within 15 significant digits, so the task is deferred (decision 0114).

## Acceptance criteria
- [ ] The 6 cells above match, and no other COSH, SINH or TANH cell in the suites changes.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7): hold-out split fixed before scoring, a fresh sweep
      drawn after the freeze, Gate A envelope result, `FITTED` tag and range if a value is fitted, the reproducibility
      floor, and no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Deferred
Phase 1b (decision 0114): every owned cell is already within 15 digits.

## Out of scope
- IMSIN/IMCOS (T-3616, T-3726).
