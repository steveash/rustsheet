---
id: T-3622
title: "Oracle sweep: CHISQ.DIST.RT and CHISQ.DIST cdf at df 3 to 100 and x from df/10 to 4 df, with a hold-out, for T-3618"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
The fit task T-3618 has dense excel-graph evidence of the chi-squared cdf (both tails) at the shapes and x where its 29
cells sit, enough for a train set and a separate hold-out (decision 0188 R4.1), so no fit is attempted blind.

## Context
- Gap triage 2026-11-01: T-3618 owns 29 CHISQ.DIST.RT, CHIDIST and CHISQ.DIST cells 1 to 6 ulp off Excel, at df 3, 5,
  10, 20, 30, 50 and 100 and x from 0.1 to 200 (probes-numeric m9-chisq-dist-rt, m9-legacy-chidist, t1049-chisqdistrt,
  m9-chisq-dist). Excel is 0 to 4 ulp off the 50-digit math at most of them and 8 to 20 at two, on either side of
  ours, so matching it is a fit. The cached evidence there is a handful of points per df. The half-integer shapes above
  a + 1 already have the 228-cell t3285 sweep (probes-numeric/t3285-chisq-dist-rt-half-integer-sweep, T-3461's
  evidence), so this sweep weights even df and x below a + 1.
- Deferred with T-3618 (decision 0114): the sweep only serves Phase 1b work, so it waits for the Phase 1 exit like the
  fit it feeds. The steward removes both keys together.
- Model: docs/research/t3285/mksweep.py (case layout, `hypotheses` labels, the mpmath side file).

## Acceptance criteria
- [ ] A new probes-numeric case `eval/suites/probes-numeric/cases/t3622-chisq-cdf-sweep.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming df and x, as in
      the t3285 case), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`
      (excel-probe skill). Grid: df in {3, 4, 5, 6, 8, 10, 15, 20, 30, 50, 75, 100}; x log-uniform in [df/10, 4 df],
      20 per df, seed stated in the description; each point as `CHISQ.DIST.RT(x,df)` and `CHISQ.DIST(x,df,TRUE)` in
      separate cells (480 cells). x is written with at most 10 significant digits so the literal is exact to read back.
- [ ] A second case `t3622-chisq-cdf-sweep-hold.case.json` (240 cells, the same grid rule with a different seed and no
      shared point) is filled in the same PR for T-3618's hold-out; its description says it is never used for fitting.
- [ ] Both `.oracle.json` sidecars are committed with no pending cells, and their math sidecars are generated with
      `rustsheet-accuracy math --suite probes-numeric` (T-3380) and committed.
- [ ] statistics.md "Incomplete gamma: accurate kernel (T-0971)" cites both cases in its Evidence, with counts of cells
      bit-exact, within 4 ulp and within 15 digits against today's engine, and of Excel's cells correctly rounded.
- [ ] probes-numeric is ungated, so the new cases add no eval/baseline/ line; `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3618).
- Half-integer shapes above a + 1 (t3285's sweep, T-3461) and the deep upper tail past x = 30 (T-3269, T-3289).
