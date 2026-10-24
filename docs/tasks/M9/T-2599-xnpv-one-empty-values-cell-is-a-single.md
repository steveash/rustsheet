---
id: T-2599
title: "XNPV: one empty values cell is a single 0 cash flow (split from T-2066)"
milestone: M9
lane: functions-financial
status: in-progress
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
- [ ] The probe cases C1-C7 pass; C8's cause is explained or recorded.
- [ ] No new known-failure lines except C8.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
