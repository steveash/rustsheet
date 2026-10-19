---
id: T-1832
title: "Probe QUOTIENT, FACTDOUBLE and SERIESSUM over a 3-D range and in legacy formulas"
milestone: M9
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The 3-D range and legacy-formula behavior of QUOTIENT, FACTDOUBLE and SERIESSUM is backed by their
own oracle cells, not only by the shared Analysis ToolPak rule.

## Context
T-1802 made these functions `LiftArrays` (SERIESSUM's coefficients `WholeByCaller`). Two of its
scalar changes follow the arrays.md rule evidenced by EDATE, WEEKNUM, EOMONTH and YEARFRAC, but are
unprobed for these functions: a 3-D range in QUOTIENT, FACTDOUBLE or SERIESSUM's x/n/m is
`#VALUE!` (FACTDOUBLE gave `#ERROR!` before), and a legacy bare range gets no `@`. Also unprobed:
`SERIESSUM(2,0,1,Sheet1:Sheet2!A1)` is `#REF!` in ours, through the coefficient reader.

## Acceptance criteria
- [ ] One probe case pins `QUOTIENT(Sheet1:Sheet2!A1,2)`, `FACTDOUBLE(Sheet1:Sheet2!A1)`,
  `SERIESSUM(Sheet1:Sheet2!A1,0,1,1)`, `SERIESSUM(2,0,1,Sheet1:Sheet2!A1)` and the FORMULATEXT of a
  legacy `=QUOTIENT(A1:A60,2)`, and ours matches it (or follow-ups are filed).
- [ ] docs/excel-semantics/numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes" drops its
  unprobed note for whatever passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' 3-D ranges.
