---
id: T-1811
title: "PERCENTILE.INC interpolation matches Excel to the last bit (PERCENTILE(C1:C6,0.3) is 1.9000000000000001)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
PERCENTILE and PERCENTILE.INC give Excel's bits where the interpolation rounds differently today.

## Context
probes/stat-argument-modes H424 `PERCENTILE(C1:C6,0.3)` and H429 `PERCENTILE.INC(C1:C6,0.3)` (C1:C6 = 1, "2", TRUE, "x", 4, blank, so the numbers are 1 and 4): Excel 1.9000000000000001, ours 1.9 (`percentile_inc_impl` in functions/statistical/percentile.rs computes `a + frac*(b-a)`; `(1-frac)*a + frac*b` is 1.9 too). Find the evaluation order Excel uses (more probe cells with two- and three-point data and several k, excel-probe skill) before changing the formula; QUARTILE.INC shares it. Named in T-1772's Outcome.

## Acceptance criteria
- [ ] H424 and H429 leave eval/baseline/probes.txt, with no new F lines in any suite.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md with its probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PERCENTILE.EXC and PERCENTRANK, unless the same probe shows them off.
