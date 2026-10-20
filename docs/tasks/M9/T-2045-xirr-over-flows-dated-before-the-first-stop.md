---
id: T-2045
title: "XIRR over flows dated before the first: stop where Excel stops on long spans (whole-column-empty-walkers E7, F3)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1591]
---

## Goal
`XIRR(A:A,B:B)` and `XIRR(A1:A1048576,B1:B1048576)` on probes/whole-column-empty-walkers return
Excel's 0.03189964592456816 (ours 0.03189964368939399, about 7e-8 relative), from the XNPV stop bound
fitted by T-1591, so no XIRR known failure is left without a task.

## Context
Needs an oracle sweep: T-1591 (two-flow XIRRs over spans of 2 to 120 years pin the form of the XNPV
part of the stop test; it has no engine work, and T-0916 is done, so nothing applies its result).
- Known failures (gap triage 2026-10-19): probes/whole-column-empty-walkers Sheet1!E7 and F3
  (excel-graph, 2026-10-07, T-1114). The flows are A1:A2 = -1000, 1100 on B1:B2 = 45000, 45365 plus
  A12:A13 = -1000, 45000 on B12:B13 = 1100, 45365 (the whole columns pick up the row-12/13 cells), so
  one flow is dated 1100, about 120 years before the first: the long-span case where
  docs/excel-semantics/financial.md "XIRR is a bracketing bisection, not Newton" says the XNPV stop
  test is not normalised by max|value| alone (xirr-early-dates A8, 124 years, Excel stops at
  bisection step 25, the model at 28).
- Engine: `compute_xirr` and `xirr_bracket` in crates/rustsheet-engine/src/functions/financial/financial_util.rs (T-1583 model);
  fit lab `uv run rustsheet-fitlab xirr` / `xirr-oos` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] The two known-failure lines (probes/whole-column-empty-walkers Sheet1!E7, Sheet1!F3) leave
      eval/baseline/probes.txt with no new lines, using the stop rule T-1591 records in financial.md.
- [ ] `rustsheet-fitlab xirr` and `xirr-oos` keep their counts or improve (record them in financial.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The zero-first-value result (xirr-early-dates A10, A11) unless T-1591's rule covers it.
