---
id: T-3400
title: "Oracle sweep: BETA.DIST cdf in the bulk at one shape 1.5 to 39 and the other 20 to 1E5 (bratio L140 bup+bgrat with several bup terms), and BETA.INV landing at the BETADIST_BETAINV G5/H5/H26 shapes"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3401 and T-3402 have excel-graph evidence, with 50-digit mpmath values beside it, of how close Excel's BETA.DIST cdf
is to correctly rounded in the bulk of the distribution where one shape is small (1.5 to 39) and the other large (20 to
1E5), and of where Excel's BETA.INV lands at the shapes of ironcalc-calc BETADIST_BETAINV H5 and H26. Today one cell
(G5) is the only Excel point in that bulk region.

## Context
- Gap triage 2026-10-31 (`make eval` on main 83d35563; `python3 scripts/exit_owners.py --all-off`): three visibly-off
  ironcalc-calc cells have no live owner. Each is "owned" only through an out-of-scope mention in deferred T-3374.
  ironcalc-calc counts for M9's exit row (82 visibly off against an allowance of 36) and for Gate B (decision 0188 R1(b)).
  - ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx Sheet1!G5 `BETA.DIST(33587,3,600,TRUE,2000,5489582)`: Excel
    0.6734553797493481 (1.3E-17 relative to the math, correctly rounded), ours 0.6734553797493648 (2.5E-14, about
    150 ulps). T-3401 owns it.
  - H5 `BETA.INV(G5,3,600,2000,5489582)` and H26 `BETA.INV(G26,1E-4,1E-4,0.15,1.2)`: T-3402 owns them.
- A scratch grid run by the triager with the engine (not committed: this task commits the evidence) puts 212 of 300
  bulk points visibly off the math (BETA.DIST(x,a,b,TRUE), x the mpmath quantile at p 0.05, 0.3, 0.5, 0.7 and 0.95,
  a in {1.5, 2, 2.5, 3, 4, 5, 8, 10, 20, 39}, b in {20, 50, 100, 600, 2000, 1E5}). The worst is 31,718 ulps
  (`BETA.DIST(2.4391745389681467E-05,2,1E5,TRUE)`: ours 0.7000000000035214, math 0.69999999999999999). Whether Excel
  is just as far off there is unknown. Where we do have Excel points in this family it is often off too, so do not
  assume it is accurate: probes-numeric/m9-beta-dist-a50 and -a100 at x = 0.1 with b from 2 to 10 have Excel 57 to 117
  ulps off the math, and the engine matches it (that is bratio's bpser branch, not L140).
- Branch, from TOMS 708's bratio as ported (crates/rustsheet-engine/src/functions/statistical/bratio/): after the
  swap a0 is the large shape and b0 the small one. With b0 < 40, b0·x0 > 0.7 and a0 > 15 the cell takes L140: bup
  for n terms, then bgrat. T-3382 (filed by T-2512, on branch agent/research/T-2512-incomplete-beta-open-pieces)
  covers one bup term. "n > 1 bup terms (2 <= b < 40)" is listed there as unprobed, and T-3383 (same branch) is the
  research home for Excel's bits there. This sweep is the evidence for both.
- Literals: write every x and p with at most 15 significant digits. Excel cuts longer literals (T-3175), so give
  the mpmath value of the literal as written, not of the quantile it came from.

## Acceptance criteria
- [ ] A new probes-numeric case `eval/suites/probes-numeric/cases/t3400-beta-cdf-bulk-several-bup.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming its inputs, as in
      `m9-fin-power.case.json`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite
      probes-numeric` (excel-probe skill), with these cells:
      - `BETA.DIST(x,a,b,TRUE)` on the grid above: a in {1.5, 2, 2.5, 3, 4, 5, 8, 10, 20, 39}, b in {20, 50, 100, 600,
        2000, 1E5}, x the 15-digit-rounded quantile at p in {0.05, 0.3, 0.5, 0.7, 0.95} (300 cells), and the mirror
        `BETA.DIST(1-x,b,a,TRUE)` written as a literal 1-x (300 cells);
      - BETA.INV landing, with Excel's cdf at its own landing point: `BETA.INV(p,3,600)` and `BETA.INV(p,3,600,2000,5489582)`
        for 20 p from 0.6 to 0.75, 0.6734553797493481 included, each beside `BETA.DIST(<that cell>,3,600,TRUE)` (or its
        bounded form); and `BETA.INV(p,s,s,0.15,1.2)` and `BETA.INV(p,s,s)` for s in {1E-4, 1E-3, 1E-2} and 10 p
        from 0.3 to 0.7, 0.4998779049300665 included, each beside the matching BETA.DIST of it.
      Keep each call at valid arity (an invalid one makes Excel for the web reject the whole workbook, T-0040).
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/research/t3400/ holds the generator script and a TSV with a 50-digit mpmath value per BETA.DIST cell (reuse
      docs/research/t2464/exact_ref.py; do not copy it) and, per (a, b), the count of cells where Excel is within
      0.5 ulp, within 4 ulp and within 15 digits of the math, and the same for the engine.
- [ ] A fixed hold-out of about 30% of the cells, chosen by a hash of the cell name, is listed in the TSV before any
      scoring (decision 0188 R4.1), for T-3402's fit.
- [ ] docs/excel-semantics/statistics.md: the T-1745 entry ("BETA.INV solves Excel's own CDF to the crossing") and the
      incomplete-beta integrated-model entry cite the case, with the counts above.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits. If the oracle is
      unavailable the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3401, T-3402).
- Modelling Excel's bits (T-3383).
