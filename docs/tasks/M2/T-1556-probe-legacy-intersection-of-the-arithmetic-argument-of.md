---
id: T-1556
title: "Probe legacy intersection of the arithmetic argument of the compatibility aggregates (STDEV, VAR, PERCENTILE, QUARTILE, MODE)"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether the compatibility aggregates (STDEV, STDEVP, VAR, VARP, PERCENTILE, QUARTILE, MODE, MODE.MULT, RANK, STDEVA, STDEVPA, VARA, VARPA, MAXIFS-style) intersect an arithmetic argument in a legacy formula, and match them.

## Context
T-1551 probed the modern aggregates (probes/legacy-value-argument-intersection-b; all intersect except MODE.SNGL) and
extended `legacy_value_operand` (parser/static_analysis/legacy.rs). The older names are unprobed and the engine keeps the
range whole. Same layout as that case; fill with excel-graph; add the ones that intersect to `legacy_value_operand`.

## Acceptance criteria
- [ ] A probe case covers the functions; references.md "Legacy formulas intersect the ranges in value arguments" lists the result; the engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
