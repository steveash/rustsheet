---
id: T-2255
title: "Sweep gen-m9 statistical, part 1 (A to L): every function under 400 Excel cells to at least 500"
milestone: M9
lane: research
status: in-progress
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
- [ ] In `make eval`'s "Excel evidence per function", every function listed above has at least
      400 Excel-backed cells, and at least 200 of them are new generated formulas with the
      function as the outer call. Aim for 500. Count from T-0062's checkpoint section once it has
      landed, and from the suite's cases until then.
- [ ] Every new cell is Excel-backed (excel-graph sidecars; the fill reports 0 failures, or the
      Outcome explains each refused case), and the suite is gated.
- [ ] `make eval-baseline` records the new failures as new coverage only: 0 regressions, and the
      decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces the new failures (as T-2114 did). Every visibly-off family gets a
      `campaign: visible` task, new or extended, listed in the Outcome with its cell count.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools, which go to the pool tasks. If a parameter has no pool, note it in the Outcome
  rather than inventing values ad hoc.

## Visible-parity stage
Coverage half of decision 0094's checkpoint: this task is done when its functions meet the
400-cell bar and their visibly-off cells all have tasks.
