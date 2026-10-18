---
id: T-1439
title: "Probe direct vs referenced and array error precedence for MEDIAN/STDEV/VAR/GEOMEAN/HARMEAN/SKEW/KURT/DEVSQ/AVEDEV/SUMSQ and PRODUCT/MIN/MAX/AVERAGE array errors"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The error order of the statistical functions built on `for_each_value` / `for_each_value_skip_empty`, and of array errors in MIN/MAX/AVERAGE, is established by oracle probes and the engine matches it.

## Context
T-1379 made `for_each_value` hold back referenced-cell errors behind direct scalar errors (evidence only for PRODUCT/MIN/MAX/AVERAGE/SUM). MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT, VAR/STDEV variants now follow it unverified; `for_each_value_skip_empty` (DEVSQ, AVEDEV, SUMSQ) still returns the first error per argument. Errors inside arrays in MIN/MAX/AVERAGE still return at once (SUM and, since T-1287, PRODUCT rank them with reference errors). Also probe a multi-cell range beside a direct error. See docs/excel-semantics/syntax.md "Aggregate error precedence".

## Acceptance criteria
- [x] Probe case with Excel values for each function above; engine matches; the UNVERIFIED note in syntax.md is resolved.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probe probes/aggregate-error-precedence-stat-family: 128 cells, excel-graph 2026-10-16, 16
  functions × 8 shapes.
- MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT, VAR, VAR.P, STDEV, STDEV.P, DEVSQ, AVEDEV, SUMSQ,
  PRODUCT, MIN, MAX and AVERAGE all rank errors like SUM: a direct error beats referenced and
  array errors, and the leftmost referenced or array error wins.
- The engine matched everywhere except DEVSQ, AVEDEV and SUMSQ, which returned the first error
  (6 cells). `SKIP_EMPTY_ARGS` now uses `ErrorRule::Ranked`.
- The UNVERIFIED note in syntax.md is resolved, and SPEC-0009's policy table is updated.
- Eval delta: probes +1 case (132 cells checked), all passing; 0 regressions.
- Follow-up: T-1696 covers VARA/VARPA/STDEVA/STDEVPA, which still use `First`, unprobed. It also
  covers a range-vs-direct shape that separates the two rules.
