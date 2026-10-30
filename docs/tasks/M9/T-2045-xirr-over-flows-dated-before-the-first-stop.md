---
id: T-2045
title: "XIRR over flows dated before the first: stop where Excel stops on long spans (whole-column-empty-walkers E7, F3)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
campaign: visible
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

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 2 in total.
- XIRR: 2 cells (13-14 digits right 0, 10-12 0, 6-9 2, fewer or 0 vs nonzero 0); most in `probes/whole-column-empty-walkers.case.json` 2. e.g. `XIRR(A:A,B:B)` Excel 0.03189964592456816, ours 0.03189964368939399

## Attempt 1 (2026-10-30, decision 0056)
- New probe probes/xirr-back-dated-stop (excel-graph, 31 cells), in `xirr-oos`. E7's flows give Excel's E7 bits as a
  literal, as a range with nine empty rows and with a zero flow dated 0 added, so the whole column is not the cause.
  25 of the 28 literal cells stop at the first iterate past the step test. The A4 family (xirr-xnpv-whole-column-nogaps A4, probe
  A4, A5, A23) takes one step more.
- Candidates (`uv run rustsheet-fitlab xirr-oos`, 81 cells; `xirr` 227):
  - T-2328's rule (the engine): 69/81 and 227/227;
  - step test only: 74/81;
  - named T-2045 candidates, the |f| bound scaled by the years from the earliest date to the latest, or from the
    first date to the farthest: 76/81 and 227/227. They miss only the A4 family and xirr-early-dates A7 (input check).
- Scratch arithmetic (269 numeric cells): no normaliser N makes one bound `c·N` fit both A4 and A6, which differ by
  one 1100 flow at the latest date. Ruled out: max|v| (scaled or not by either span), Σ|v|, the discounted terms at m,
  |XNPV'| (bare, times |m| or times 1 + m), XNPV at the bracket ends, at 0, at the first or the previous midpoint,
  |Σv| and the positive sum. A step test on the signed m: 218/280, against 269.
- Not applied: in the engine the earliest-to-latest span fixes E7, F3 and the probe's E7 family (10 cells) but
  regresses nogaps A4.
- Next hypothesis (attempt 2): what sets the A4 family apart. It has three flows with the back-dated one last and a
  negative root. Probe these one at a time (the back-dated flow moved earlier in the list, a fourth flow at the first
  date, positive-root twins) and fit the |f| test on the cells where it bites.
- This task now also owns the probe's 12 known failures (probes/xirr-back-dated-stop A1-A3, A6-A12, A17, A18).
- Found in review: XIRR reads numeric text in its ranges as numbers (probe A31, ours `#VALUE!`), recorded in
  financial.md and filed as T-3267, which owns A31.
- Eval delta: none fixed; 13 new-coverage known failures (the new probe: 12 owned here, A31 by T-3267),
  0 regressions.
