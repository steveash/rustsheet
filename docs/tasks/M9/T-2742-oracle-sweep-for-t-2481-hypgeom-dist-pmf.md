---
id: T-2742
title: "Oracle sweep for T-2481: HYPGEOM.DIST pmf and cdf over every small population (N 6 to 12)"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel values for HYPGEOM.DIST over every valid (x, n, K, N) with a population from 6 to 12, as pmf and cdf. Each cell
is given its offset in ulps from the exact rational (computed by the analysis script, not stored in the case). T-2481
can then pin Excel's pmf formulation from cases where the candidates disagree.

## Context
Gap triage 2026-10-25 (`make eval` on `main` 22588cf). HYPGEOM.DIST and HYPGEOMDIST are a few ulps off the exact
rational almost everywhere, and visibly off it at a few cells. docs/excel-semantics/statistics.md has the detail in
"HYPGEOM.DIST is not exact for small integer arguments".
- T-2481 (todo, opus) owns the visibly-off ones. Excel vs ours, where ours is the exact value rounded:
  - gen-m9/f2bd5f380ec002bf A52 `HYPGEOMDIST(2,2,3,11)`: 0.05454545454545458 vs 0.05454545454545454 (3/55).
  - gen-m9/ee2497a5e683c5d9 A310 `HYPGEOMDIST(2,4,3,19)`: 0.09287925696594419 vs 0.09287925696594428.
  - probes-numeric/t1049-hypgeomdist A10 `HYPGEOM.DIST(5,10,25,100,FALSE)`: 0.052973714479659324 vs
    0.05297371447965936.
- The last-digit tail on the same run: 51 HYPGEOMDIST and 10 HYPGEOM.DIST gen-m9 cells. For example,
  gen-m9/0a614cfe974b59c3 A168 `HYPGEOMDIST(2,4,9,16)` is 0.4153846153846155 in Excel and 0.4153846153846154 for us.
- T-0977 rejected these formulations. None reproduces more than 25 of the 84 cached small-population pmf cells bit
  for bit:
  - exp of the nine log-factorials in all 9! summation orders (correctly rounded, lgamma, or summed logs);
  - ratios of COMBINs;
  - a product of ratios walked from x = 0;
  - Loader's `dhyper`.
- T-2481's Context says "Needs evidence beyond the sweeps (excel-probe skill)": a grid at a fixed small N, so the
  formulation can be pinned from exactly representable cases such as `HYPGEOMDIST(0,3,1,6)` = 0.5000000000000001.
  No task had filed that grid.

This is a numeric-fitting group that needs more Excel points than the cached workbooks give. Following the
gap-triager's rule, it is an oracle sweep, not a blind fit. T-2481 depends on this task (the filing PR added
the dependency).

Sweep design: one case, `eval/suites/probes-numeric/cases/t2742-hypgeom-small-n.case.json`, tag `M9`,
`"expected": {}`. Follow the `excel-probe` skill and write the layout into the case description. Generate it with a
script, and commit the script or record its command in the description.
- For every N from 6 to 12, every n from 1 to N-1, every K from 1 to N-1, and every x from max(0, n+K-N) to min(n, K):
  - column A: `=HYPGEOM.DIST(x,n,K,N,FALSE)`;
  - column B: `=HYPGEOM.DIST(x,n,K,N,TRUE)`.
  That is about 1,600 rows.
- Column C: `=HYPGEOMDIST(x,n,K,N)` for N = 12 only, to confirm that the legacy name shares the kernel.
Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`; the oracle reported
excel-graph OK on 2026-10-25. If no backend is available, leave the case pending and set this task `blocked` on the
oracle.

Analysis (scratch or fit lab):
- Compute each cell's exact rational with Python `fractions`, and its signed ulp offset from Excel's value.
- Tabulate the offsets by N and by x position: the first term, interior terms, and the last term.
- Re-run T-0977's rejected candidates on the grid and record each one's bit-exact count. A new fitlab kernel
  (`hypgeom`) is welcome but not required.
- Check that every cdf cell is the running double sum of Excel's own pmf cells from the lowest x.

## Acceptance criteria
- [x] The case and its `.oracle.json` (excel-graph, canary-verified) are committed with the layout above.
      probes-numeric is ungated (`gate = false`, no file in eval/baseline/), so `make eval` reports the case's cells
      and no baseline line changes.
- [x] statistics.md "HYPGEOM.DIST is not exact for small integer arguments" gains a T-2742 paragraph. It gives the
      share of cells on the exact value, the ulp-offset pattern with cited cells, each candidate's bit-exact count on
      the grid, and whether the cdf-as-sum-of-pmfs rule holds on every cdf cell.
- [x] T-2481's Context points at the new evidence (it already lists this task in `depends_on`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change, or choosing the formulation (T-2481).
- Populations past 2E9 and the `#NUM!` limit (T-2429, T-2699), and negative fractional counts (T-2700).

## Outcome
Committed probes-numeric/t2742-hypgeom-small-n (1442 rows, 3291 cells, excel-graph 2026-10-28). statistics.md gained
the T-2742 paragraph: 286/1442 pmf cells exact, offsets from -14 to +10 ulps at every x position, n/K asymmetry (434/1442
symmetric), candidate bit-exact counts (best 286, the exact rational itself), the cdf equal to the running pmf sum in
816/1442 cells and 1 at the last x in all 476, and HYPGEOMDIST = HYPGEOM.DIST in 407/407. T-2481's Context points at the
evidence. The suite is ungated, so no baseline changed. The analysis is docs/research/t2742_hypgeom_grid.py (exact
rationals from Python `fractions`, signed ulp offsets per cell; it prints every number above).
