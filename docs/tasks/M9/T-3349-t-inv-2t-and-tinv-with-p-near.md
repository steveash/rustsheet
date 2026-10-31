---
id: T-3349
title: "T.INV.2T and TINV with p near 1 lose the small quantile to the 1-p cancellation"
milestone: M9
lane: functions-stat
status: done
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
- Overlap: T-2724 (accuracy, todo), T-2839 (T.INV.2T and TINV just above p = 1) and T-2858 (df 1) own existing
  cells of these functions; T-2724 owns the existing visibly-off centre-quantile cells (62 cells in its file); this
  task adds new probe cells and the fix. Whichever lands first re-triages the others (T-2724,
  T-2839, T-2858) in its Outcome. T-3363 replaces the
  inverse algorithm as a whole; this task is the user-visible bug and goes first.

- Probe (T-3349): probes/t3349-t-inv-2t-near-one A1-A44 (excel-graph, 2026-10-31): A1 the assessment's
  `T.INV.2T(0.999999999989522,2)`, Excel 1.481806159731988E-11; A2 TINV, A3 df 1, A4 df 30, A5 CONFIDENCE.T of it;
  A6-A35 `T.INV.2T(1-1E-k,df)` for k 3, 6, 9, 12, 15 at df 1, 2, 5, 30, 1000, 1E6; A36-A38 p = 1.001, 1.000000001, 1
  (the 16-digit 1+1E-15 reads as 1) at df 2; A39-A44 T.INV at 0.5 +- 1E-6, 1E-12 and TINV(1-1E-6, 1-1E-12) at df 2.
  The bug the assessment saw (1.49E-8 at A1) was fixed on main by T-2724 (#2510) before this task started; the probe
  is the evidence that it stays fixed. Sweep: probes-numeric/t3349-t-inv-2t-sweep (300 points, seed 3349).
## Acceptance criteria
- [x] `T.INV.2T(0.999999999989522,2)` is within 15 digits of Excel's value (a unit test and a probe cell). When the
      probe lands, record Excel's filled value for it in this file's Context and in statistics.md.
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [x] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
- The bug was fixed on main by T-2724 (#2510, the centre crossing `student_t_centre`) before this task started:
  `T.INV.2T(0.999999999989522,2)` is 1.4818061597319863E-11, Excel 1.481806159731988E-11 (was 1.49E-8). This task
  adds the evidence: a unit test (t_dist.rs `centre_quantiles_keep_their_digits`, the assessment's input and probe
  A15, A29 at Excel's values), the probe and the fresh sweep, and the statistics.md entry.
- Probe probes/t3349-t-inv-2t-near-one (44 cells, filled): all 44 within 1E-9 of the math; 37 within 15 digits of Excel.
  The 7 others: A3, A6-A10 are df 1, Excel's 1/TAN form (T-2858 owns them; up to 2.9% off the math at A10), and A5
  `CONFIDENCE.T(0.999999999989522,1,3)` is Excel's last digits (Excel 1.1E-15 off the math, ours on it; Phase 1b,
  decision 0188 R5). Recorded as new-coverage known failures in eval/baseline/probes.txt (40 cells not bit-exact).
- Gate A envelope on the fresh sweep probes-numeric/t3349-t-inv-2t-sweep (300 points, seed 3349, scored against
  mpmath at 60 digits by a scratch script; SPEC-0029's generator is T-3357), per stratum, points within 15 digits of
  Excel / within 1E-9 of the math / outside both / error against number:
  typical 41 / 120 / 0 / 0 of 120; tail 69 / 90 / 0 / 0 of 90; extreme 22 / 60 / 0 / 0 of 60; log-uniform 6 / 30 / 0 / 0
  of 30. Of the 162 points not within 15 digits of Excel: 110 have Excel on the math and ours off (median 5E-15,
  at most 5E-13 relative: the typical-input error the Goal names, which this task does not lower; T-3363 replaces the
  inverse), 10 have Excel 1E-13 to 1E-4 off (its rounded-y form, T-3061), and 42 have Excel more than 1E-4 off the
  math at p below 1.73E-110 with df from 2490 (filed as T-3386 for an owner ruling under decision 0188 R3).
- Eval delta: probes +1 case (44 cells, 4 bit-exact, 0 regressions); probes-numeric +1 case (gate-free).
- Follow-up: T-3386.
