---
id: T-3288
title: "Oracle sweep: POISSON.DIST pmf at large k (30 to 1000) with the mean from k/2 to 2.5k, and ulp runs at the five visibly-off cells"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3295 can fit Excel's POISSON.DIST pmf at large k against a dense excel-graph sweep, instead of the 27 cells with
k >= 30 that probes-numeric/t2055-poisson-pmf-sweep holds today (A178-A204: k = 30, 100 and 300 only, nine means each).

## Context
- T-2055 (done) fitted the pmf to two closed forms (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed
  forms"): the small-mean closed form, and Loader's saddle point exp(-stirlerr(k) - bd0(k, mean)) / sqrt(2 pi k)
  (`statistical/poisson.rs`, `loader.rs` `excel_bd0`) from mean 1.1. At large k and a mean well off k, five cells are
  visibly off, and Excel itself is not correctly rounded there (50-digit mpmath, ulp of Excel's value; gap triage
  2026-10-30, `make eval` on origin/main 73b187224):

  | cell (probes-numeric/t2055-poisson-pmf-sweep) | formula | Excel | ours | Excel vs exact | ours vs exact |
  |---|---|---|---|---|---|
  | Calc!A193 | `POISSON.DIST(100,140.0,FALSE)` | 6.943404181610098e-05 | 6.943404181610092e-05 | +21.6 | +16.6 |
  | Calc!A196 | `POISSON.DIST(300,210.0,FALSE)` | 9.509204270307778e-10 | 9.509204270307811e-10 | -3.8 | +12.2 |
  | Calc!A197 | `POISSON.DIST(300,240.0,FALSE)` | 2.222766347231691e-05 | 2.2227663472316755e-05 | +10.0 | -36.0 |
  | Calc!A200 | `POISSON.DIST(300,330.0,FALSE)` | 0.005638971394817704 | 0.005638971394817709 | -2.0 | +4.0 |
  | Calc!A201 | `POISSON.DIST(300,375.0,FALSE)` | 7.297037209220307e-06 | 7.297037209220347e-06 | +2.6 | +49.6 |

- Excel is the inaccurate side, so the fit has to reproduce Excel's own arithmetic (bd0's series and how
  stirlerr + bd0 is stored). Nine means per k do not pin that, and T-3247 found that a fully stored bd0 series wins at
  (300,210) and (300,240) but loses (300,420). The same `excel_bd0` sits under BETA.DIST's density for min(a,b) > 1
  (T-2586, T-3220, T-3244), so these values help those fits too.

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t3282-poisson-pmf-large-k.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in
      `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid:
      - `POISSON.DIST(k, mean, FALSE)` for k in {30, 50, 75, 100, 150, 200, 300, 500, 1000} and mean = k * r for r in
        {0.5, 0.6, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 1.05, 1.1, 1.15, 1.2, 1.25, 1.3, 1.4, 1.5, 1.75, 2, 2.5}
        (171 cells; write each mean as its decimal literal);
      - ulp runs at the five cells above: `POISSON.DIST(k, m+j*2^-e, FALSE)` for j from -16 to 15, with 2^-e the ulp
        of m (2^-45 for m = 140, 210, 240; 2^-44 for m = 330, 375), so 160 cells of adjacent doubles.
      Keep each call at valid arity (an invalid one makes Excel for the web reject the whole workbook, T-0040), and
      split into two cases if the oracle's size limits need it.
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms" cites the new case in its Evidence line,
      with the count of cells that are bit-exact with today's engine (from `make eval-show`).
- [ ] T-3295 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3295). The cdf (T-3269, T-3289). The small-k residue (T-3247).
