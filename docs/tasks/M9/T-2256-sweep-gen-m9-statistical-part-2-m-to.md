---
id: T-2256
title: "Sweep gen-m9 statistical, part 2 (M to Z): every function under 400 Excel cells to at least 500"
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
The M9 statistical functions M to Z under 400 Excel cells reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 62 in all:
  MAXA (72), MAXIFS (69), MEDIAN (47), MINA (86), MINIFS (65), MODE (92), MODE.MULT (50), MODE.SNGL (54), NEGBINOM.DIST (108), NEGBINOMDIST (32), NORM.INV (323), NORMDIST (161), NORMINV (28), NORMSDIST (152), NORMSINV (242), PEARSON (45), PERCENTILE (42), PERCENTILE.EXC (29), PERCENTILE.INC (30), PERCENTRANK (45), PERCENTRANK.EXC (42), PERCENTRANK.INC (56), PERMUT (60), PERMUTATIONA (25), POISSON (47), POISSON.DIST (99), PROB (48), QUARTILE (37), QUARTILE.EXC (46), QUARTILE.INC (45), RANK (104), RANK.AVG (40), RANK.EQ (28), RSQ (45), SKEW (48), SKEW.P (32), SLOPE (45), SMALL (60), STANDARDIZE (28), STDEV (79), STDEV.P (88), STDEV.S (110), STDEVA (50), STDEVP (53), STDEVPA (44), STEYX (74), T.DIST.2T (208), T.INV (286), T.INV.2T (293), T.TEST (109), TDIST (48), TINV (53), TREND (191), TRIMMEAN (29), TTEST (54), VAR (108), VARA (54), VARP (88), VARPA (44), WEIBULL (40), WEIBULL.DIST (260), ZTEST (60).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M9 --focus --category statistical --functions <the M-Z list> --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`.

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
gen-m9 grew from 95 to 163 cases and from 28,862 to 56,062 scored cells, all excel-graph-backed (fills: 38 + 30
sidecars, 0 failures, no case refused). The suite was already gated. Generator commands:
- `cd tools && uv run rustsheet-gen --milestone M9 --focus --category statistical --functions <the 62 M-Z functions> --count 38 --formulas 400 --seed 51`.
- Top-ups: the 38 functions still under 400 Excel cells, `--count 25 --formulas 400 --seed 52`; the 17 between 400
  and 500, `--count 5 --formulas 400 --seed 53`.

Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`. After the sweep every one
of the 62 functions has at least 535 Excel cells (minimum STDEV.S 535, T.TEST 540, MAXIFS 541) and is the outer call
of at least 243 new generated formulas (minimum T.INV.2T and WEIBULL.DIST). gen-m9 is 87.38% bit-exact;
eval/baseline/gen-m9.txt went from 5,132 to 7,085 known failures, all 1,953 new coverage, 0 regressions in every
baseline. Visible parity: 1,393 of the new failures are visibly off; the decision 0088 lock went 6,854 -> 7,703 (+849,
not +1,393, because fixes merged on main had brought the count below the old lock; new coverage only). Evidence floor
stays 0.

Mismatch families (gap-triager, traced to the innermost diverging sub-expression; visibly-off cells (all cells)):
- New: T-2468 PROB argument/validation order, 115 (115); T-2475 residual inner sub-expressions to probe, 83 (83);
  T-2469 T.TEST error order, 64 (64); T-2472 order statistics: scalar argument's error first and PERCENTRANK's
  #N/A before significance, 62 (62); T-2474 extreme arguments part 2, 42 (42); T-2471 MODE family one-cell empty
  reference, 36 (36); T-2470 Z.TEST data rules, 35 (35); T-2473 PERCENTRANK huge significance, 9 (11);
  T-2476 MEDIAN/TRIMMEAN/SKEW/STDEV last digits, 0 (29).
- Extended: T-0967 TINV/T.INV/T.INV.2T, 133 (234); T-2430, 105 (105); T-2426, 81 (81); T-2344, 72 (72); T-2425,
  63 (63); T-2055 POISSON, 50 (133); T-2096, 48 (48); T-1726 TREND, 47 (47); T-1390, 45 (45); T-2146 NEGBINOMDIST,
  40 (85); T-2428, 39 (39); T-2239 WEIBULL, 29 (44); T-1849, 28 (28); T-0969 TDIST/T.TEST, 19 (247); T-2424,
  19 (19); T-2432, 18 (18); T-2431, 17 (17); T-2093, 17 (17); T-1811 PERCENTILE, 13 (42); T-2427, 12 (12);
  T-1741, 11 (11); T-2429, 11 (11); T-1743, 10 (10); T-1742, 9 (9); T-1634, 5 (5); T-0974, 2 (2); T-2242, 1 (2);
  T-2185, T-2363, T-0972, 1 (1) each; T-0975 NORMDIST, 0 (16); T-0925 NORMINV, 0 (11). Total 1,393 (1,953).
Quirks recorded in docs/excel-semantics/statistics.md (PROB, T.TEST and Z.TEST argument order, MODE on an empty
one-cell reference, order-statistic error order, PERCENTRANK significance, extreme arguments part 2, T quantile at
p=0). Also filed T-2467 (TEXTJOIN lifted-edge probes, from the T-2318 reviews). No edge pool was missing.
