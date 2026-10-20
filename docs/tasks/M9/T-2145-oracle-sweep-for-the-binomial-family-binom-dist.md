---
id: T-2145
title: "Oracle sweep for the binomial family: BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST pmf and cdf on short-decimal p"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
T-2146 can fit Excel's binomial evaluation against a dense excel-graph sweep, not only the 148 desktop cells of
ironcalc-calc STATISTICAL/BINOM.xlsx that T-0927 already exhausted.

## Context
- T-0927 (done) made the engine sum the exact pmf terms and round once. That matches Excel at 78 of 148 BINOM.xlsx
  cells, comes within 4 ulp at 57 and within 4E-14 relative at 13 (statistics.md "Discrete distributions: within an
  ulp of the exact sum"). Excel is sometimes one ulp off an exact short decimal: `BINOM.DIST(3,10,0.5,TRUE)` is
  0.17187500000000006 and `BINOMDIST(3,10,0.5,FALSE)` 0.11718750000000003 (probes-numeric/m9-legacy-binomdist A5,
  A7). `NEGBINOM.DIST(0,100,0.01,TRUE)` is `exp(100*ln(0.01))` in doubles. These values point to `exp`/`ln` forms
  or an incomplete-beta route, and telling them apart needs many points where the exact value is a short decimal.
- Existing web cases: probes-numeric/m9-legacy-binomdist (54 cells, 21 failing) and t1048-negbinomdist (40 cells,
  12 failing).

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t2145-binomial-sweep.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as
      in `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid:
      - BINOM.DIST(k, n, p, FALSE) and (k, n, p, TRUE) for n in {1..12, 15, 20, 30, 50, 100}, every k from 0 to n
        (all k for n <= 20, about 10 spread values above), p in {0.5, 0.25, 0.125, 0.1, 0.2, 0.3, 0.7, 0.9, 0.99,
        0.999}.
      - NEGBINOM.DIST(f, r, p, FALSE/TRUE) for r in {1, 2, 3, 5, 10}, f in 0..10, the same p.
      - BINOM.DIST.RANGE(n, p, s, s2) for n <= 12 and a few (s, s2) pairs per n.
      - Keep each call at valid arity. An invalid one makes Excel for the web reject the whole workbook (T-0040).
        Keep the workbook under the oracle's size limits; split it into two cases if needed.
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] statistics.md "Discrete distributions" records what the sweep shows, citing the case: where Excel is exact,
      where it is one ulp off a short decimal, and any p or n pattern.
- [ ] T-2146 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-2146). POISSON.DIST (T-2055). HYPGEOM.DIST (T-0977, T-1896).
