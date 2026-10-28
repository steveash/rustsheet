---
id: T-2481
title: "HYPGEOM.DIST: Excel's pmf formulation, where its own value is visibly off the exact one"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0977, T-2742]
campaign: visible
---

## Goal
HYPGEOM.DIST and HYPGEOMDIST reproduce Excel's pmf where Excel's own value is more than half a unit of the 15th digit
off the exact rational, so the last visibly-off HYPGEOM cells pass.

## Context
New evidence (T-2742): probes-numeric/t2742-hypgeom-small-n has Excel's pmf and cdf for every valid (x, n, K, N) with
N 6-12. statistics.md "HYPGEOM.DIST is not exact for small integer arguments" summarizes it: 286 of 1442 pmf cells are
exact, offsets reach 14 ulps, the pmf is not symmetric in n and K, the cdf is not the running sum of the pmf, and
HYPGEOMDIST shares the kernel.
T-0977 made the pmf and cdf the exact value rounded once (`discrete.rs` `hypergeometric_walk`). Excel is a few ulp off
the exact value everywhere, and visibly off it at these cells (Excel vs ours = exact):
- gen-m9/f2bd5f380ec002bf A52 `HYPGEOMDIST(2,2,3,11)`: 0.05454545454545458 vs 0.05454545454545454 (3/55)
- gen-m9/ee2497a5e683c5d9 A310 `HYPGEOMDIST(2,4,3,19)`: 0.09287925696594419 vs 0.09287925696594428
- gen-m9/60b0b6e69ad35a45 A140 (a HYPGEOM.DIST of Data cells): 3.936600103532592E-16 vs 3.936600103532582E-16
- probes-numeric/t1049-hypgeomdist A10 `HYPGEOM.DIST(5,10,25,100,FALSE)`: 0.052973714479659324 vs 0.05297371447965936
- A14 `HYPGEOM.DIST(50,100,500,10000,FALSE)`: 7.89124343483521E-39 vs 7.891243434835164E-39
- A16 `HYPGEOM.DIST(100,1000,5000,100000,FALSE)`: 3.482317936286154E-11 vs 3.482317936286112E-11
- Large populations (gap triage 2026-10-27; they had no owner): probes/t2429-hypgeom-huge-count A7 `HYPGEOMDIST(1,2,3,1000000)`
  5.999987999988011E-06 vs 5.999987999988E-06, A25 `HYPGEOMDIST(1,2,3,1000000000)` 5.999999987999986E-09 vs
  5.999999988E-09; probes/t2429-hypgeom-threshold A1 `HYPGEOMDIST(1,2,3,1073741824)` 5.58793543728452E-09 vs
  5.58793543728453E-09, A9 `HYPGEOMDIST(1,2,3,2000000000)` 2.9999999970000066E-09 vs 2.999999997E-09.
What T-0977 ruled out (statistics.md "HYPGEOM.DIST is not exact for small integer arguments"): exp of the nine
log-factorials in all 9! orders (correctly rounded, lgamma, summed logs), ratios of COMBINs, a product of ratios from
x = 0, Loader's `dhyper`. Each matches at most 25 of 84 small-population pmf cells. Excel's cdf is the sum of its own
pmf terms, so the pmf formulation fixes the cdf's last digits too.
**Needs evidence beyond the sweeps (excel-probe skill):** e.g. a grid at a fixed small N (N = 6 to 12, every n, K,
x) to pin the formulation from exactly representable cases such as `HYPGEOMDIST(0,3,1,6)` = 0.5000000000000001.

Bit-level scope left by T-0977 (decision 0083), which the same formulation should settle: ironcalc-calc
STATISTICAL/HYPERGEOM_DIST.xlsx F2, F3; the 65 last-digit HYPGEOM gen-m9 lines in T-0977's task file; and
t1049-hypgeomdist A5 `HYPGEOM.DIST(0,3,5,12,TRUE)` (Excel 0.15909090909090923, now the exact 7/44, 5 ulp off; it was 2
ulp off under statrs).

## Acceptance criteria
- [ ] The ten cells above are within half a unit of the 15th digit of Excel's value, no new known-failure lines.
- [ ] statistics.md records the matched formulation; the bit-level cells above pass or are listed with Excel vs ours.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The population limit near 2^53 (statistics.md, same entry).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the visibly-off cells of HYPGEOMDIST (5) and HYPGEOM.DIST (2). Fixing them lifts both to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells. Its dependency T-2742 is promoted with it (decision 0094).
