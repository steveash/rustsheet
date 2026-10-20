---
id: T-1975
title: "Match last-ulp results of VDB (fin-vdb-modes A6, A7, A10, A13), MDURATION and RRI"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VDB, MDURATION and RRI return Excel's exact doubles on the cells that differ only in the last ulp.

## Context
Split from T-1764. Failing cells: probes/fin-vdb-modes A6 (203.31648000000007), A7 (203.3164800000004, depends on no_switch), A10 (2099.9999999999995), A13 (157.28640000000001); probes/fin-mduration A1-A6, A8, A17; probes/fin-rri A14. Closed-form and book-value variants of the VDB schedule were tried and none reproduced the Excel doubles; the operation order needs fitting against the oracle values. Code: `vdb_total_depr` in functions/financial/financial_depreciation.rs.

## Acceptance criteria
- [ ] those cells pass and leave eval/baseline/probes.txt
- [ ] `make check` green; reviewers approve.

## Out of scope
-
