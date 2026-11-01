# T-3509: the gamma family's held-out probe

A permanent, ungated, refreshable hold-out for GAMMA.DIST, CHISQ.DIST.RT, GAMMA.INV, CHISQ.INV.RT, POISSON.DIST,
GAMMALN and GAMMA (decision 0188 R4.2): 4,100 cells drawn log-uniform over each function's whole domain, blind to
every fit, filled from Excel and from 50-digit math.

## Rules

1. **Never fit to this probe.** No constant, cut-point, table, form selector or branch region may be chosen,
   tuned or checked against these cells while it is being chosen, and no fit-lab sweep may read them. A fit that
   has seen them has spent them: refresh the probe (below) before the next fit is judged.
2. **Every fit change in the gamma family is scored on it before and after.** Run the report on main and on the
   branch, and put both per-function tables (and the regions that moved) in the task's Outcome, next to the fit's
   R4.1 hold-out counts. A change that loses cells here, against Excel or against the math, has to explain them.
3. Once a branch has been scored on it and merged, these points have been seen by a reviewer, not fitted; they stay
   a hold-out until a fit is chosen with them in view. If in doubt, refresh.

## Files

| Path | What |
|---|---|
| `docs/research/t3509/sweep.py` | The generator (seed `SEED`; `USED_SEEDS` lists every seed already filled). |
| `eval/suites/probes-numeric/cases/t3509-blind-*.case.json` | The six cases, sheet `S` (columns in each case's description). |
| `…/t3509-blind-*.oracle.json` | Excel's values (`rustsheet-oracle fill -b excel-graph`, 2026-10-31). |
| `…/t3509-blind-*.math.json` | 50-digit mpmath values (`rustsheet-accuracy math`, definitions in `tools/src/rustsheet_tools/accuracy/defs/gamma.py`); the runner attaches them, so R1(c) and Gate A's envelope see them. |
| `docs/research/t3509/score.py` | The per-region report. |
| `docs/research/t3509/score-main-744815d06.out` | The report for main 744815d06, with every visibly-off cell (`--cells`). |

The cases are the points of the 2026-10-31 blind sweep at main 3003a898b (same seed and draw order). The refill
on 2026-10-31 gave the same Excel value at all 4,100 cells as that sweep's fill.

The first draw's GAMMA.INV case has a quirk: where 1 − t rounds to 1 (t below 1.1E-16) the sweep wrote p = 0.5, so 227
of its 500 cells are the median. Every visibly-off cell in its "shape below 0.5, p in the upper half" region is one of
them (`GAMMA.INV(0.5, a, 1)`). Later seeds redraw t there instead.

## Scoring a branch

```
flock /tmp/rs-eval.lock cargo build --release -p rustsheet_eval    # then copy the binary and use the copy
BIN=/tmp/<dir>/rustsheet-eval
$BIN run --suite probes-numeric --filter t3509-blind- --out /tmp/<dir>/run
cd tools && uv run python ../docs/research/t3509/score.py /tmp/<dir>/run/report.json [--cells]
```

## Reading the report

Each line is a region: the function (GAMMA.DIST and POISSON split into cdf and pdf/pmf), a shape bucket (the gamma
shape a; df/2 for chi-squared; k + 1 for Poisson; x itself for GAMMALN and GAMMA) and a second bucket: x/a for the
distributions (x/df for CHISQ.DIST.RT, mean/(k + 1) for POISSON), p for the quantiles (CHISQ.INV.RT's p is its right
tail). `n` is the region's cell count. Three comparisons follow, each in four exclusive classes:

- `=` bit-exact, `4u` within 4 ulp, `15d` within half a unit of the 15th significant digit, `off` visibly off (or a
  different type or error);
- **ours vs Excel** is the target (decision 0083: `off` is a visible-parity miss);
- **ours vs math** compares against the double nearest the 50-digit value;
- **Excel vs math** says how accurate Excel itself is there. Where Excel is `=`/`4u` against the math and we are not,
  the gap is an `accuracy` one (a better kernel); where Excel is itself far from the math it is `visible` (Excel's
  own arithmetic, time-boxed, decision 0056);
- `outA` counts our cells outside Gate A's envelope (neither within 15 digits of Excel nor within 1e-9 of the math;
  an error against a number either way); `nomath` the cells with no math value.

The per-function lines at the end are the totals to quote before and after a change.

## Refreshing

When a fit has been chosen with these cells in view, or once a quarter, draw new points:

1. Pick a seed not in `USED_SEEDS` (e.g. the date, `20270115`); add it to `USED_SEEDS` and set `SEED` to it.
2. `python3 -I docs/research/t3509/sweep.py` (rewrites the six cases).
3. `cd tools && RUSTSHEET_EVAL_BIN=<copied binary> uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric
   --case probes-numeric/t3509-blind-<name>.case.json` for each case (`--force` replaces the old values).
4. `uv run rustsheet-accuracy math --suite probes-numeric --filter t3509-blind-`.
5. Score main, commit the new `score-main-<sha>.out` (remove the old one) and file tasks for new regions.

## Findings at main 744815d06

Per function, ours vs Excel (`=` / `4u` / `15d` / `off`), and outside Gate A:

| Function | = | 4u | 15d | off | outA |
|---|---|---|---|---|---|
| GAMMA.DIST cdf | 358 | 79 | 19 | 44 | 0 |
| GAMMA.DIST pdf | 300 | 149 | 10 | 41 | 0 |
| CHISQ.DIST.RT | 421 | 48 | 7 | 24 | 0 |
| GAMMA.INV | 215 | 207 | 23 | 55 | 1 |
| CHISQ.INV.RT | 311 | 188 | 1 | 0 | 0 |
| POISSON cdf | 393 | 53 | 14 | 40 | 0 |
| POISSON pmf | 428 | 53 | 5 | 14 | 0 |
| GAMMALN | 297 | 3 | 0 | 0 | 0 |
| GAMMA | 296 | 4 | 0 | 0 | 0 |

The one cell outside Gate A is `GAMMA.INV(0.5,7990464.8,1)`, `#NUM!` where Excel has a number (fixed elsewhere).
Every other visibly-off cell is within Gate A, and in every region below Excel is itself 6 to 228 ulp (median) from
the math, so they are `visible` gaps. Where they are and who owns them:

| Region | Off | Owner |
|---|---|---|
| GAMMA.DIST cdf/pdf, shape 2.5–20, x/a below 0.5 | 25 + 23 | T-3219 (evidence added; T-3412 on T-3311's branch is the same form) |
| GAMMA.DIST cdf/pdf elsewhere: shape 0.5–2.5 at x/a below 1E-6, shape 20–1E3 at x/a below 0.5, single cells | 19 + 18 | T-3617 |
| GAMMA.INV median (p = 0.5) at shape below 0.5 | 34 | T-3519 |
| GAMMA.INV deep lower tail (p below 1E-10), shape 0.5–20 | 16 | T-3520 (after T-3311; T-3403 inside T-3311's region) |
| GAMMA.INV other: two lower-tail cells below shape 0.5, one at shape 20–1E3 (T-3294), one at 2.5–20 upper, the `#NUM!` | 5 | — |
| CHISQ.DIST.RT, shape 20–1E3, x/df 1.4–10 | 8 | T-3521 |
| CHISQ.DIST.RT far upper tail at shape below 20 | 16 | T-3310's region (df 3–22) |
| POISSON cdf, mean 1.4 times k + 1 and above (deep upper tail) | 35 | T-3269's region |
| POISSON cdf near the mean (0.5–1.4) | 5 | T-3618 |
| POISSON pmf, mean 0.5–1.4 times k + 1, k above 20 (12 above 1000) | 14 | T-3522 (T-3295 and T-3247 stop at k = 1000) |
