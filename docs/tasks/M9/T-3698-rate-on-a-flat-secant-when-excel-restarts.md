---
id: T-3698
title: "RATE on a flat secant: when Excel restarts from the older iterate instead of holding"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
A rule, fitted on oracle evidence, for when RATE's secant restarts from the older iterate on a flat secant (f1 = f0)
instead of holding at the newer one, so that probes/t3579-rate-residual-nper1 A1, A2, A41 and A42 and
t3348-rate-fv-nonzero A171 match Excel without losing the held cells.

## Context
- T-3579 (financial.md "RATE holds on a flat secant and runs on below -1", the "residual is not the cause" bullet):
  at nper 1 Excel's residual has the model's bits (49/53 bit-exact). The 4 misses are flat secants the model holds,
  where Excel's value is the model's run restarted from the flat step's older iterate
  (`RATE(1,2560574.93,-2543613.41,-11634.76,0,0.0020941704345913)`, A6, is Excel's A1 bit for bit; scale 4, A42,
  fits too).
- A blanket restart loses 32 held cells of t3348-rate-fv-nonzero (and 31 Gate A sweep cells; A43/A46 here), so
  Excel holds on some flat secants and restarts on others. Candidates to test: the iterate count when the flat
  secant occurs (A171's comes at step 3, right after the first near-root step), whether the flat step's two
  iterates differ (zero step vs distinct iterates), |f| against a bound, a restart budget (once per run), or the
  restart point (guess vs older iterate).
- Code: `compute_rate` (crates/rustsheet-engine/src/functions/financial/financial_util.rs); fit lab `rate`
  (tools/src/rustsheet_tools/fitlab/kernels/solvers.py). A structural model chosen among ranked candidates owes the
  hold-out and fresh sweep (decision 0188 R4).
- Known failures owned: probes/t3579-rate-residual-nper1 Sheet1!A1, A2, A41, A42.

## Acceptance criteria
- [ ] A probe that separates the restart from the hold (e.g. flat secants at different steps and |f|, with and
      without a prior restart), oracle-filled, and a rule that fits it.
- [ ] `compute_rate` implements the rule; the 4 cells above pass and no RATE cell regresses
      (t3348-rate-fv-nonzero, the Gate A sweeps, t3478-rate-r4-wide); R4 hold-out and fresh sweep recorded.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- RATE's residual form (ruled out at nper 1 by T-3579).
