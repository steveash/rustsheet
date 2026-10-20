---
id: T-1927
title: "F.TEST last-digit accuracy and error-cell paired size order"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
F.TEST/FTEST match Excel to the last digit, and the order of the size check against a range holding an error cell is probed.

## Context
After T-1292, probes/stat-argument-modes H248 and H251 (`F.TEST(C1:C6,B1:B8)`) are 2 ulps off (0.9195979047175997 vs 0.9195979047175995), and H246/H247/H249/H250 are NumericClose; the F cdf path is in functions/statistical/fisher.rs. Separately, `fn_get_two_matrices` checks sizes before reading cells, so CORREL of an error-holding range against a range of another size is `#N/A`; no oracle evidence decides it (use the excel-probe skill with an equal-size and an unequal-size pair where one side holds `#DIV/0!`).

## Acceptance criteria
- [ ] H248 and H251 pass exactly (or the ulp tolerance is justified in docs/excel-semantics/statistics.md); the error-cell order is probed and recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
