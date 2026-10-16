---
id: T-1439
title: "Probe direct vs referenced and array error precedence for MEDIAN/STDEV/VAR/GEOMEAN/HARMEAN/SKEW/KURT/DEVSQ/AVEDEV/SUMSQ and PRODUCT/MIN/MAX/AVERAGE array errors"
milestone: M2
lane: engine
status: in-progress
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
- [ ] Probe case with Excel values for each function above; engine matches; the UNVERIFIED note in syntax.md is resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
