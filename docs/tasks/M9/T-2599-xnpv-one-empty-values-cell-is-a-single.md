---
id: T-2599
title: "XNPV: one empty values cell is a single 0 cash flow (split from T-2066)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`XNPV(1,A2,B11)` with A2 empty is 0, as Excel (probes/xnpv-empty-values-cell C1-C4); `XNPV(1,A2:A3,B11:B12)` (both values empty) is `#VALUE!` (C8, known failure).

## Context
Split from T-2066 (reviewer-architecture: one task, one lane). Branch agent/functions-stat/T-2066-quartile-text-cell carries a working fix in `get_array_of_numbers_xpnv` (functions/financial/mod.rs), the probe, and the financial.md entry. Land this first: gen-m9/eccd03df42e40006 A24 (`PERCENTRANK(<no numbers>,XNPV(..))`, Excel `#N/A`) must pass before T-2066's PERCENTRANK reorder, or that reorder regresses it. Cherry-pick those pieces from the T-2066 branch.

## Acceptance criteria
- [x] The probe cases C1-C7 pass; C8's cause is explained or recorded.
- [x] No new known-failure lines except C8.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`get_array_of_numbers_xpnv` takes an explicit `XnpvArgument` (values or dates) instead of keying on the error. One empty
cell, `A2` or the one-cell range `A2:A2`, is a single 0 cash flow as values and `#NUM!` as dates
(probes/xnpv-empty-values-cell C1-C7, new probes/xnpv-empty-cell-forms C1-C5, excel-graph 2026-10-24). C8
(`XNPV(1,A2:A3,B11:B12)`, Excel `#VALUE!`, ours `#NUM!`) stays a recorded known failure, unexplained (financial.md).
Eval: gen-m9 known failures -3, probes +1 known failure (C8, new case), 0 regressions.
