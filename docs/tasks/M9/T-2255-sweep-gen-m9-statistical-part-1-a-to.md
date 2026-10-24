---
id: T-2255
title: "Sweep gen-m9 statistical, part 1 (A to L): every function under 400 Excel cells to at least 500"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063, T-2247]
---

## Goal
The M9 statistical functions A to L under 400 Excel cells reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 65 in all:
  AVEDEV (59), AVERAGEA (156), AVERAGEIF (126), AVERAGEIFS (89), BETADIST (49), BETAINV (54), BINOM.DIST (106), BINOM.DIST.RANGE (61), BINOM.INV (61), BINOMDIST (45), CHIDIST (55), CHIINV (62), CHISQ.DIST (154), CHISQ.INV (107), CHISQ.INV.RT (144), CHISQ.TEST (46), CHITEST (27), CONFIDENCE (33), CONFIDENCE.NORM (74), CONFIDENCE.T (144), CORREL (45), COUNTBLANK (195), COVAR (35), COVARIANCE.P (49), COVARIANCE.S (48), CRITBINOM (34), DEVSQ (58), EXPON.DIST (93), EXPONDIST (45), F.DIST (117), F.DIST.RT (113), F.INV (97), F.INV.RT (118), F.TEST (46), FDIST (73), FINV (60), FISHER (41), FISHERINV (61), FORECAST (53), FORECAST.ETS (22), FORECAST.ETS.CONFINT (19), FORECAST.ETS.SEASONALITY (10), FORECAST.ETS.STAT (14), FORECAST.LINEAR (46), FREQUENCY (21), FTEST (28), GAMMA.DIST (299), GAMMA.INV (208), GAMMADIST (39), GAMMAINV (27), GAMMALN.PRECISE (60), GAUSS (254), GROWTH (50), HARMEAN (43), HYPGEOM.DIST (95), HYPGEOMDIST (34), INTERCEPT (47), KURT (43), LARGE (80), LINEST (45), LOGEST (38), LOGINV (26), LOGNORM.DIST (235), LOGNORM.INV (116), LOGNORMDIST (26).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M9 --focus --category statistical --functions <the A-L list> --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`.
- FORECAST.ETS* sit below the 20-cell evidence floor; give them their own run, with data arrays long enough for seasonality detection.
## Acceptance criteria
- [x] In `make eval`'s "Excel evidence per function", every function listed above has at least
      400 Excel-backed cells, and at least 200 of them are new generated formulas with the
      function as the outer call. Aim for 500. Count from T-0062's checkpoint section once it has
      landed, and from the suite's cases until then.
- [x] Every new cell is Excel-backed (excel-graph sidecars; the fill reports 0 failures, or the
      Outcome explains each refused case), and the suite is gated.
- [x] `make eval-baseline` records the new failures as new coverage only: 0 regressions, and the
      decision 0088 lock moves only by new coverage.
- [x] The gap-triager traces the new failures (as T-2114 did). Every visibly-off family gets a
      `campaign: visible` task, new or extended, listed in the Outcome with its cell count.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools, which go to the pool tasks. If a parameter has no pool, note it in the Outcome
  rather than inventing values ad hoc.

## Visible-parity stage
Coverage half of decision 0094's checkpoint: this task is done when its functions meet the
400-cell bar and their visibly-off cells all have tasks.

## Outcome
gen-m9 grew from 25 to 95 cases and from 998 to 28,862 scored cells, all excel-graph-backed (fills: 43 + 27 sidecars,
0 failures, no case refused; 36 cells holding an undecodable rich-value error were left pending by the oracle, owned by
T-2397). The suite is now gated (`gate = true`). Generator commands:
- `cd tools && uv run rustsheet-gen --milestone M9 --focus --category statistical --functions <the 61 non-ETS> --count 38 --formulas 400 --seed 41`:
  each function is the outer call of about 250 formulas.
- FORECAST.ETS family on its own: `--functions FORECAST.ETS,FORECAST.ETS.CONFINT,FORECAST.ETS.SEASONALITY,FORECAST.ETS.STAT --count 5 --formulas 400 --seed 42`.
  The generator has no long sorted timeline pool, so these are mostly argument-error cells (excel-graph never computes
  the family anyway, T-1293); pool gap below.
- Top-ups: the 46 functions still under 400 cells, `--count 25 --formulas 400 --seed 43`; the 7 between 400 and 500
  (AVERAGEA, AVERAGEIF, AVERAGEIFS, CHISQ.DIST, CHISQ.INV.RT, CONFIDENCE.T, GAMMA.INV), `--count 2 --formulas 350 --seed 44`.

Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`. After the sweep every one
of the 65 functions has at least 500 Excel cells (minimum LOGNORM.DIST 515; the four FORECAST.ETS functions 536-554)
and at least 320 generated outer-call cells. gen-m9 is 82.19% bit-exact; eval/baseline/gen-m9.txt records 5,147 known
failures, all new coverage (the 25 earlier cases were never baselined), 0 regressions in every baseline. Visible
parity: 4,291 of them are visibly off (lock 2,947 -> 6,851, new coverage only, decision 0088). The lock rose by 3,904,
not 4,291: `make eval-baseline` counted 6,851 = 4,291 in the new cases + 2,560 in cases already baselined, which is 387
below the old lock (fixes merged on main that no baseline update had recorded yet; a count below the lock passes). After merging main
the count is 6,801; the steward's daily `make eval-lower-visible-lock` records that drop (decision 0093). Evidence floor 3 -> 0
(the three FORECAST.ETS functions under 20 cells; decision 0010 entry).

Mismatch families (gap-triager, traced to the innermost diverging sub-expression; visibly-off cells):
- New: T-2424 FORECAST.ETS argument errors, text and lifting (1,196), T-2425 FORECAST.ETS values from desktop Excel
  (877, blocked on T-1553/T-2424), T-2427 FREQUENCY error arguments and bins+1 rows (139), T-2436 probe of untraced
  inner sub-expressions (116), T-2426 empty one-cell reference `#VALUE!` in the paired functions and F.TEST (109),
  T-2428 XIRR/XNPV size and type checks (83), T-2429 HYPGEOMDIST bounds (29), T-2435 scalar distributions at extreme
  arguments (20), T-2430 error element in an array argument (19), T-2431 paired functions pair by count, not shape (17),
  T-2432 KURT/F.TEST over overflowing data (11), T-2433 lift cost cap `#ERROR!` where Excel spills (9), T-2434 GAUSS
  of a tiny argument (8).
- Extended: T-2093 F.TEST/FTEST (409), T-1742 LOGEST (167), T-1741 LINEST (159), T-1634 (136), T-1871 (118),
  T-0974 (96), T-1849 (90), T-0971 (68), T-2361 (66), T-1743 (65), T-0972 (64), T-1390 (46), T-2219 (42), T-0967 (35),
  T-1891 (25), T-0977 (22), T-1538 (16), T-0925 (14), T-2146 (8), T-1749 (4), T-1726 (2), and one cell each in
  T-1864, T-2066, T-2119, T-2044, T-0975, T-2005.

Total 4,291 visibly off. The 856 last-digit cells went to existing accuracy tasks (T-1749 169, T-2221 137, T-0972 121,
T-2361 119, T-0971 82, T-0977 65, T-0974 51, T-0925 47, T-2146 21, T-0973 6, T-0967 5, T-2006 3, T-0975 3, T-0968 3,
T-2119 2) and the new T-2437 (HARMEAN, EXPON.DIST/EXPONDIST, FISHER/FISHERINV, 22). Quirks recorded in
docs/excel-semantics/statistics.md and financial.md.

Pool gaps (no edge pool; noted, not invented; filed as T-2438): integer counts (BINOM.DIST/BINOMDIST/BINOM.DIST.RANGE `number_s`,
`trials`, `number_s2`; BINOM.INV/CRITBINOM `trials`; the HYPGEOM.DIST/HYPGEOMDIST counts; CONFIDENCE* `size`; LARGE `k`),
the FORECAST.ETS options and a long sorted timeline, BETADIST/BETAINV `A`/`B`, FISHERINV `y`, FORECAST/FORECAST.LINEAR
`x`, and the lognormal `mean`.
