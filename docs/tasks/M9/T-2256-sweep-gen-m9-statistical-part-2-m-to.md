---
id: T-2256
title: "Sweep gen-m9 statistical, part 2 (M to Z): every function under 400 Excel cells to at least 500"
milestone: M9
lane: research
status: todo
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
