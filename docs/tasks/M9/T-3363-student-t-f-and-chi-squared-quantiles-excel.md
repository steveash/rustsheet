---
id: T-3363
title: "Student-t, F and chi-squared quantiles: Excel's own inverse algorithm in place of the probe-tuned statrs inverse-beta port"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3360]
campaign: visible
---

## Goal
TINV, T.INV, T.INV.2T, CONFIDENCE.T, F.INV, FINV, F.INV.RT and CHISQ.INV compute their quantiles with Excel's actual
inverse algorithm, identified from evidence, instead of the statrs inverse-beta port with probe-tuned thresholds, and
the result generalizes to fresh points.

## Context
- Owner's 2026-10-31 assessment (decision 0188): on fresh stratified points these functions generalize badly (share
  within 15 digits, fresh against public): T.INV 30% vs 69%, T.INV.2T 14% vs 51%, F.INV 63% vs 78%. The cause is the
  statrs inverse-beta port (statistical/inverse_beta.rs) with probe-tuned thresholds (t_dist.rs `T_QUANTILE_SERIES_DF`,
  `T_TAIL_P`, `T_TAIL_X`, `T_STATRS_MIN_P`) and tuned form selectors. Functions that port Excel's algorithm (ERF,
  NORM.*, GAMMALN, Bessel) generalize.
- Evidence to start from: docs/excel-semantics/statistics.md "Student-t quantile is SQRT(df*(1/y-1)) of the rounded beta
  quantile" (T-3061), "F.INV is a crossing of F.DIST's tails" (T-0974), "BETA.INV solves Excel's own CDF to the
  crossing" (T-1745), "Student-t quantile at one degree of freedom is 1/TAN(π·p/2)" (T-2858), "Chi-squared quantile at
  df 2 ... closed forms" (T-2860); docs/research/excel-kernel-playbook.md.
- Overlap: T-2511 (crossing of the ported incomplete beta), T-0967 (T.INV parity), T-3061, T-2859, T-2724 and T-1542
  own existing cells of these functions. This task identifies and ships the algorithm; its Outcome re-triages those
  tasks (closing what it settles). The deep-tail quantiles are `MATH-OVER-EXCEL` (decision 0188 R3): keep the math there.
- T-3349 (P0) fixes the T.INV.2T cancellation first; build on it.

## Acceptance criteria
- [ ] The algorithm (start, iteration, stop rule) is identified with evidence and recorded in statistics.md; no statrs
      inversion or probe-tuned threshold is left for these functions (`rg` shows it), or each one left carries a
      `FITTED(...)` tag inside its measured range with an accurate fallback outside (decision 0188 R4.3).
- [ ] Out-of-sample generalization shown: the fit lab's hold-out split (T-3360; train and test counts side by side,
      R4.1) and a fresh log-uniform sweep over the full domain filled after the model is frozen (R4.2), each function's
      fresh within-15-digit share within a stated margin of its public share.
- [ ] The Gate A envelope holds on that sweep (decision 0188 R2): no point outside both 15 digits of Excel and 1e-9 of
      the math, and no error/number mismatch.
- [ ] No gated cell regresses beyond decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity on cells already within 15 digits (Phase 1b, decision 0188 R5).

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when the owned cells are within half a unit of the 15th significant digit of
Excel's value (math within ε on `MATH-OVER-EXCEL` points) and the fresh sweep shows the envelope. Time-boxed by decision
0056.
