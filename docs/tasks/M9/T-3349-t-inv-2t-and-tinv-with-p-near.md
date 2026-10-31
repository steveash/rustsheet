---
id: T-3349
title: "T.INV.2T and TINV with p near 1 lose the small quantile to the 1-p cancellation"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
T.INV.2T, TINV and the two-tailed form CONFIDENCE.T uses return the small quantile without the 1 − p cancellation:
`T.INV.2T(0.999999999989522,2)` gives 1.49e-8 today against a true 1.48e-11 (5 orders of magnitude). The typical-input
error of T.INV and T.INV.2T against the math (median 7.5e-15 and 1.5e-14, Excel 3e-16 and 6e-16) falls with it.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 3). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_t_inv_2t`, `fn_tinv` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs and the statrs
  inverse-beta port in inverse_beta.rs. Near p = 1 the two-tailed quantile is small, so solve on the centre directly
  (for example through the incomplete beta's complement in its small argument) instead of forming 1 − p/2.
- Overlap: T-2724 (accuracy, todo) owns the existing visibly-off centre-quantile cells (62 cells in its file); this
  task adds new probe cells and the fix. Whichever lands first re-triages the other in its Outcome. T-3363 replaces the
  inverse algorithm as a whole; this task is the user-visible bug and goes first.

## Acceptance criteria
- [ ] `T.INV.2T(0.999999999989522,2)` is within 15 digits of Excel's value (a unit test and a probe cell).
- [ ] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [ ] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [ ] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number.
- [ ] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.
