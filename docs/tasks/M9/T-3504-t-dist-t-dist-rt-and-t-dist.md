---
id: T-3504
title: "T.DIST, T.DIST.RT and T.DIST.2T generalize poorly to fresh points (88% within 15 digits against 98% on the public cases): find the failing strata and fix the kernel"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The Student-t CDF family (T.DIST cumulative and density, T.DIST.RT, T.DIST.2T, and TDIST through them) is within 15
digits of Excel on fresh stratified points at about the rate it reaches on the public cases, with the cause of the gap
found and fixed in the kernel, not tuned to the sweep.

## Context
- Decision 0188 (owner's 2026-10-31 assessment, "Fresh stratified points"): T.DIST is within 15 digits on 88% of 120
  fresh stratified points against 98% on the public cases, one of the six functions that "generalize badly"; the
  cause named is "a statrs inverse-beta port with probe-tuned thresholds (inverse_beta.rs, t_dist.rs) and tuned form
  selectors". T-3346 filed owners for the quantiles (T-3363) and for POISSON.DIST and BINOM.DIST (T-3366) but none
  for the Student-t CDF; gap triage 2026-10-31 found no open task that measures it (`rg 'fresh stratified'
  docs/tasks` lists only T-3363 and T-3366).
- The assessment's points are not committed, so this task starts with its own fresh sweep (decision 0188 R2 strata:
  about 40% typical inputs, 30% tails and domain edges, 20% extreme but valid, 10% log-uniform; df 1 to 1E10 with
  fractional dfs, which Excel truncates; x from 1E-300 to 1E300 and both signs), oracle-filled (excel-graph) with a
  50-digit mpmath value per point, as T-3349 did (probes-numeric/t3349-t-inv-2t-sweep, docs/research/t3354/sweep.py).
- Code: statistical/student_t.rs (the ported kernel and its form selectors: `SERIES_MAX_DF`, `LOG_GAMMA_BETA_MIN`
  and `LOG_GAMMA_BETA_MAX` tagged `FITTED(T-0969, ...)`, `KERNEL_MIN_DF` `FITTED(T-2509, ...)`, `ROUNDED_Y_MAX`
  `FITTED(T-0969, 3232 cells, df 2..1000, t 0.0013..8)`, `ASYMPTOTIC_MIN_SHAPE`), statistical/t_dist.rs.
  Several of these were fitted on df 2 to 1000 and t below 8 (their tags give the range), so a fresh point outside
  that range is the first suspect (decision 0188 R4.3).
- Related, with their own cells: T-3364 (extrapolated fits fall back to accurate math; it names the Student-t df
  bands, so if the failing strata are exactly those bands, ship the fix there and close this task with the evidence),
  T-3365 (re-validating tuned selectors, quality lane), T-3079 (routing the T-0969 sweep cells), T-0969 (time-boxed),
  T-2509 and T-2585 (tails and density). This task owns no existing eval cell; it owns the fresh-point gap.

## Visible-parity stage
`visible`: where the fresh sweep shows Excel on the math and the engine off, the fix is a more accurate kernel; where
Excel is the inaccurate one, the target is still Excel's value (directive 1), time-boxed (decision 0056). Target: the fresh within-15-digit share of each function within a stated margin of its public share.

## Acceptance criteria
- [ ] A fresh stratified sweep per function (T.DIST TRUE, T.DIST FALSE, T.DIST.RT, T.DIST.2T; about 120 points
      each, fixed seed) is committed in probes-numeric, excel-graph filled, with mpmath values attached; statistics.md
      records the failing strata (which df, x and form-selector band) and their cause, against Excel and against the
      math.
- [ ] After the fix, a second fresh sweep filled after the change (R4.2) shows each function's within-15-digit share
      within a stated margin of its public share; the margin is stated before scoring.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any changed constant, cut-point, band or
      selector: hold-out counts (chosen against correctly rounded, train and test side by side, a manual ~30% split by
      cell hash until T-3360 lands); the fresh sweep's result; the Gate A envelope result on both sweeps with every
      point outside listed; `FITTED(T-3504, n_cells, range)` tags applied only inside their range, with accurate math
      outside; no fit below the oracle's reproducibility floor (4 ulp); no keep-matched branch.
- [ ] No gated cell regresses beyond decision 0083's allowance (`make eval-baseline` refuses none).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Student-t quantiles (T-3363, T-2858, T-3061).
- T.TEST beyond what the shared kernel changes.
