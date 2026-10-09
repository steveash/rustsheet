---
id: T-1290
title: "Statistical functions lift scalar arguments over arrays and ranges (probes/stat-argument-modes)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every scalar argument of the M9 statistical functions lifts elementwise over an array and over a multi-cell range, as in Excel.

## Context
probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): every one of the 254 `ROWS(F(..{v;v}..))` and `ROWS(F(..G1:G2..))` cells is 2 in Excel (distribution
parameters, LARGE/SMALL k, PERCENTILE/QUARTILE k, PERCENTRANK and Z.TEST x, RANK number, T.TEST tails/type, TRIMMEAN
percent, FORECAST x, PROB lower_limit, the FORECAST.ETS target_date and statistic_type). The engine is `#N/IMPL!` for an
array (207 cells) and `#VALUE!` for a range (89 cells). Reuse the lifting the math and date functions already use (search
before adding a helper); BETA.DIST, Z.TEST and ZTEST are `#VALUE!` even for an array. Split by family if larger than M.

## Acceptance criteria
- [ ] The lift cells of probes/stat-argument-modes pass (their baseline lines removed).
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
