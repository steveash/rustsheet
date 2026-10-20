---
id: T-1975
title: "Match last-ulp results of VDB (fin-vdb-modes A6, A7, A10, A13), MDURATION and RRI"
milestone: M9
lane: functions-financial
status: done
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
- [x] those cells pass and leave eval/baseline/probes.txt
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probes, excel-graph filled: probes/fin-vdb-schedule (78 VDB cells: integer and fractional windows, factors,
  lives, both modes) and probes/fin-mduration-rri-fit (60 cells: DURATION, MDURATION and the worksheet quotient;
  RRI and its `^` form).
- VDB (financial.md "VDB charge order"): Excel sums the window's per-period charges.
  - no_switch TRUE: `min(b·rate, b-salvage)` with `b = cost·(1-rate)^(i-1)` raised directly.
  - Switching from a whole start: iterated book value, declining-balance charges summed, plus `n·sl` for the
    straight-line periods.
  - Fractional-start switching keeps the old model: A42-A46 of the new case stay known failures, owned by T-2185
    (the lab's best restart model, 89/92, is recorded there).
- DURATION uses Excel's `^` and `(t·cf)/v^t`; MDURATION is DURATION/(1+y/f) as before. RRI is `(fv/pv)^(1/n)-1` with
  Excel's operators.
- Eval: all 14 target cells pass (fin-vdb-modes 20/20, fin-mduration 18/18, fin-rri 16/16); new cases 73/78 and 60/60.
