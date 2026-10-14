---
id: T-1551
title: "Probe legacy intersection of the arithmetic argument of the remaining aggregates (MIN, PRODUCT, STDEV, VAR, SUMSQ, ...)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know which other aggregates intersect an arithmetic argument in a legacy formula, and match them.

## Context
T-1526 (docs/excel-semantics/references.md "Legacy formulas intersect the ranges in value
arguments", probes/legacy-value-argument-intersection) showed that legacy `SUM(B1:B6+0)`,
`AVERAGE`, `AVERAGEA`, `MAX`, `COUNT`, `COUNTA`, `LARGE` and `MEDIAN` of `B1:B6*1` intersect the
range (`#VALUE!` off its rows), while MMULT, INDEX and LOOKUP read such an argument as an array.
The engine applies it only to those eight (`legacy_value_operand` in parser/static_analysis.rs).
Write one probe case (same layout: B1:B6 = 1,2,3,1,2,3, each formula on row 2 and row 10, listed in
`legacy_formulas`, `O10 = B1:B6*1` as the control) for MIN, MINA, MAXA, PRODUCT, SUMSQ, STDEV.S,
STDEV.P, VAR.S, VAR.P, DEVSQ, AVEDEV, GEOMEAN, HARMEAN, SMALL, PERCENTILE.INC, QUARTILE.INC, MODE.SNGL,
KURT, SKEW, GCD, LCM, CONCAT and TEXTJOIN (third argument), fill it with excel-graph, then add the
ones that intersect to `legacy_value_operand` with the case as evidence.

## Acceptance criteria
- [ ] The probe case is filled and passes; `make eval-baseline` records it.
- [ ] references.md lists the result per function.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
