---
id: T-1551
title: "Probe legacy intersection of the arithmetic argument of the remaining aggregates (MIN, PRODUCT, STDEV, VAR, SUMSQ, ...)"
milestone: M2
lane: engine
status: done
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
KURT, SKEW, GCD, LCM, CONCAT and TEXTJOIN (third argument); also `SUM((B1:B6*1))`, `SUM(B1:B6>1)`,
`SUM(B1:B6&"")`, `SUMPRODUCT(ABS(IFERROR(B1:B6,100)))`, `MAXA`/`MINA(XLOOKUP(..))` over a text cell,
dynamic `AVERAGEA({1,"a",TRUE})` and `AVERAGEA("1/2/2020",1)` (engine choices from T-1526 with no
oracle cell yet), and FORMULATEXT of one legacy `SUM(B1:B6+0)`; fill it with excel-graph, then add the
ones that intersect to `legacy_value_operand` with the case as evidence.

## Acceptance criteria
- [x] The probe case is filled and passes; `make eval-baseline` records it.
- [x] references.md lists the result per function.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `legacy-value-argument-intersection-b` (64 cells, excel-graph): every probed aggregate intersects an
arithmetic operand in a legacy formula except MODE.SNGL, and so do `SUM((B1:B6*1))`, `SUM(B1:B6>1)` and
`SUM(B1:B6&"")`. `legacy_value_operand` now covers MIN, MINA, MAXA, PRODUCT, SUMSQ, STDEV.S/P, VAR.S/P, DEVSQ,
AVEDEV, GEOMEAN, HARMEAN, SMALL, PERCENTILE.INC, QUARTILE.INC, KURT, SKEW, GCD, LCM, CONCAT and TEXTJOIN, and
sees through parentheses (`is_value_operation`). Results per function in references.md. Left as a known failure:
dynamic `AVERAGEA({1,"a",TRUE})` (Excel 0.5, we 0.667), filed as T-1555. Eval delta: 64 new cells, 63 pass, 1 known failure.
