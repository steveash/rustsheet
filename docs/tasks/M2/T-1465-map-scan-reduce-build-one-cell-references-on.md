---
id: T-1465
title: "MAP/SCAN/REDUCE: build one-cell references on demand and read single cells without an array"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MAP, SCAN and REDUCE over a range use O(1) extra memory for the cell references and read a one-cell reference result without allocating, with unchanged results.

## Context
Performance review of T-1442 (all MINOR): `lambda_cells` (bycol_byrow.rs) materialises a dense grid of up to 2^20 `CalcResult::Range` (about 48 MB per argument) when `(range.left, i, j)` would do; `lambda_result_value` goes through `result_to_array` for a 1x1 reference, where `evaluate_cell` would not allocate; REDUCE reads its accumulator on every step, while a reference could be carried to the end if Excel evidence allows (needs a probe of `ISREF(a)` in a REDUCE accumulator).

## Acceptance criteria
- [ ] The probes/lambda-helper-range-slices cells stay passing; `REDUCE(0,A:A,..)` and `MAP(A:A,..)` allocate no per-cell grid.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
