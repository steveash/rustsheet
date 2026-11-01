---
id: T-3295
title: "POISSON.DIST pmf at large k and a mean well off k: Excel's saddle point on the 5 visibly-off t2055 cells (needs the T-3288 sweep)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3288]
campaign: visible
---

## Goal
`POISSON.DIST(k,mean,FALSE)` and `POISSON(k,mean,FALSE)` at a large k (30 and up) with a mean well off k give Excel's
value to 15 significant digits, starting with the five t2055 cells that are visibly off today, without losing any
Excel-computed pmf cell that passes.

## Visible-parity stage
`visible`: Excel is not correctly rounded here (2 to 22 ulp from the 50-digit value), so the target is Excel's own
saddle-point arithmetic, not a more accurate kernel. Target: the five cells listed below within half a unit of the 15th
significant digit, with no cell of any suite regressing. Time-boxed (decision 0056). Bit-level misses left after that go
to T-3247 (`defer: phase-1b`), which keeps the rest of the pmf residue.

## Context
- Needs an oracle sweep: the 27 cells with k >= 30 in probes-numeric/t2055-poisson-pmf-sweep (k = 30, 100, 300, nine
  means each) cannot separate the candidate forms, so this task waits for T-3288's dense sweep (k = 30 to 1000, mean
  k/2 to 2.5k, and ulp runs at the five cells). Do not start a fit before its sidecar lands.
- T-2055 (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms"; `statistical/poisson.rs`) uses
  Loader's saddle point exp(-stirlerr(k) - bd0(k, mean)) / sqrt(2 pi k) from mean 1.1, with `loader.rs` `excel_bd0`.
  T-3247 (deferred) owns this pmf's bit-level residue. This task takes its five visibly-off cells, so the visible part
  is no longer parked behind `defer: phase-1b` (`scripts/exit_owners.py --all-off` lists them as owned only by a deferred
  task). T-3247's suspects: `excel_bd0`'s series (a fully stored series gets (300,210) and (300,240) but loses
  (300,420)) and the stored exponent E = fl(-stirlerr - bd0), which Excel's binomial pmf holds extended
  (statistics.md "BINOM.DIST pmf").
- Cells (gap triage 2026-10-30, `make eval` on origin/main 73b187224; ulp against 50-digit mpmath, in ulps of Excel's
  value):
  - `POISSON.DIST(100,140.0,FALSE)` (A193): Excel 6.943404181610098e-05 (+21.6), ours 6.943404181610092e-05 (+16.6)
  - `POISSON.DIST(300,210.0,FALSE)` (A196): Excel 9.509204270307778e-10 (-3.8), ours 9.509204270307811e-10 (+12.2)
  - `POISSON.DIST(300,240.0,FALSE)` (A197): Excel 2.222766347231691e-05 (+10.0), ours 2.2227663472316755e-05 (-36.0)
  - `POISSON.DIST(300,330.0,FALSE)` (A200): Excel 0.005638971394817704 (-2.0), ours 0.005638971394817709 (+4.0)
  - `POISSON.DIST(300,375.0,FALSE)` (A201): Excel 7.297037209220307e-06 (+2.6), ours 7.297037209220347e-06 (+49.6)
- The same `excel_bd0` is under BETA.DIST's density for min(a,b) > 1 (T-2586, T-3220, T-3244): change it only behind a
  Poisson-specific path unless those tasks' cells also hold.

Cells this task owns (split from T-3247):
- probes-numeric/t2055-poisson-pmf-sweep.case.json (5): Calc!A193, A196, A197, A200, A201

## Acceptance criteria
- [ ] The five cells above are within half a unit of the 15th significant digit of Excel (`make eval-show
      CASE=probes-numeric/t2055-poisson-pmf-sweep`), and the T-3288 sweep's cells are scored and reported in the
      Outcome (bit-exact / within 15 digits / visibly off, before and after).
- [ ] No Excel-computed cell of POISSON, POISSON.DIST, BINOM.DIST, NEGBINOM.DIST or BETA.DIST goes from pass to fail
      (main-vs-HEAD per cell); no new line in eval/baseline/ (`make eval-baseline` refuses regressions).
- [ ] docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms" records the form found (or the forms
      rejected, with their scores on the sweep).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The small-k pmf residue and `POISSON_STIRLERR_2_TO_5` (T-3247). The cdf (T-3269).

T-3288 landed the sweep: probes-numeric/t3282-poisson-pmf-large-k.case.json (171 cells) and
probes-numeric/t3282-poisson-pmf-ulp-runs.case.json (160 cells), both filled by excel-graph.
