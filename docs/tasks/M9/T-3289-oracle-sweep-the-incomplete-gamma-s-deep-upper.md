---
id: T-3289
title: "Oracle sweep: the incomplete gamma's deep upper tail, POISSON cdf at mean 4(k+1) to 100(k+1) and CHISQ.DIST.RT at x 4 to 100 df"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3269 can fit Excel's deep upper tail of the regularized incomplete gamma Q(a, x) (x well above a, a below 20) against
a dense excel-graph sweep, instead of the dozen scattered gen-m9 and probe cells it has today.

## Context
- POISSON(k,mean,TRUE) = Q(k + 1, mean) and CHISQ.DIST.RT(x,df) = CHIDIST(x,df) = Q(df/2, x/2) (statistics.md "One
  incomplete-gamma kernel ... (T-1778)"). Where x is well above a, Excel is 10 to 405 ulp from the 50-digit value and
  ours is within about 3 (gap triage 2026-10-30, `make eval` on origin/main 73b187224; ulp of Excel's value):
  `POISSON(2,100,TRUE)` +34.2 (gen-m9/c7664e494e418395 A294), `POISSON(1,100,TRUE)` -12.1 (gen-m9/097af1ca33638670
  A214), `POISSON(9,100,TRUE)` = `CHISQ.DIST.RT(200,20)` -43.3 (gen-m9/5919660b7bb4d195 A80, probes-numeric/t1049-chisqdistrt
  A28), `POISSON(8,42,TRUE)` +22.6, `POISSON(6,180,TRUE)` -25.3, `CHISQ.DIST.RT(200,5)` -32.3, `CHISQ.DIST.RT(1000,3)`
  +86.7, `CHIDIST(1000,5)` -43.1, `CHIDIST(1000,30)` +405.0 (probes-numeric/m9-legacy-chidist A71).
- statistics.md "Incomplete gamma: accurate kernel (T-0971)" already records that no summed-log prefactor tried
  reproduces these (T-2638 fit attempt 1), and T-3269 that 48 storage orders of exp(fl(a LN x - x - GAMMALN(a))) times
  the series fail. A fit needs many points along each axis: shape, x / a, and adjacent doubles.
- T-3252 sweeps POISSON's cdf only for mean / k from 0.1 to 3, near the mode; this sweep is the far tail, mean / (k + 1)
  from 4 to 100.

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t3284-gamma-deep-upper-tail.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in
      `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid:
      - `POISSON(k, mean, TRUE)` for k in {0, 1, 2, 3, 4, 5, 6, 8, 10, 13, 16, 19} and mean = (k + 1) * r for r in
        {4, 5, 6, 8, 10, 15, 20, 30, 50, 100}, skipping a mean above 600 (the value underflows there);
      - for each of those cells its twin `CHISQ.DIST.RT(2*mean, 2*(k+1))`;
      - `CHISQ.DIST.RT(x, df)` for the half-integer shapes, df in {1, 3, 5, 7, 9, 15, 21, 31, 39}, and x = df * s for
        s in {4, 6, 8, 10, 20, 50, 100}, skipping an x above 1200;
      - ulp runs: `POISSON(2,100+j*2^-46,TRUE)` and `CHISQ.DIST.RT(200+j*2^-45,5)` for j from -16 to 15 (64 cells of
        adjacent doubles).
      Keep each call at valid arity (an invalid one makes Excel for the web reject the whole workbook, T-0040), and
      split into two cases if the oracle's size limits need it.
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" cites the case in its Evidence
      line, with the count of cells where Excel is within 1 ulp of the 50-digit value (any mpmath script used goes under
      docs/research/t3284/).
- [ ] T-3269 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3269). The near-mode window (T-3252, T-3286). Shapes 1.5 and 2.5 at x up to about 13a,
  where Excel is accurate (T-3285).

## Outcome
One probes-numeric case, `t3284-gamma-deep-upper-tail` (343 cells, filled by excel-graph, no pending cells; ulp-run means are
written as `m+(j)*2^(-e)` since a literal is read to 15 digits). Excel is within 1 ulp of the 50-digit value in 64 cells
(`docs/research/t3284/ulp_vs_exact.py`); the engine is bit-exact on 53. Documented in statistics.md; T-3269 links the case.
No eval baseline change (probes-numeric is ungated).
