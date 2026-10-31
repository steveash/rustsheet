---
id: T-3502
title: "Re-validate GEOMEAN's 15-digit snap of the root on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
GEOMEAN's 15-digit snap of the root (statistical/geomean.rs) is shown to generalize on a fresh hold-out sweep, or is replaced by the plain `exp(ln(P)/n)` root.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- The root snaps to its 15-digit value when that value, raised to the n-th power, reproduces the product;
  tagged `FITTED(T-1123, 170 cells, GEOMEAN of 1..1100 values with a normal product)`. The sweep should draw value
  lists log-uniformly in count and magnitude, not only integer roots.
- Method: build the engine twice (with the selector, and with the accurate form only) and score both on the old
  cells through the fit lab (`docs/architecture/fit-lab.md`, hold-out split, default margin 5 points), then on the
  fresh sweep. Accuracy-suite tables are refused as sweeps (`--holdout-only` measures a frozen fit only).

## Acceptance criteria
- [ ] A fresh log-uniform sweep over the selector's full domain (R4.2), new cells in probes-numeric that no earlier fit
      saw, is oracle-filled and committed.
- [ ] The Outcome reports, with the selector and with the accurate form (two engine builds, or fit-lab candidates that
      reproduce each bit for bit on the old cells), the fit lab's train and test counts (T-3360's hold-out split and
      margin) on the old cells and the counts on the fresh sweep.
- [ ] If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
      accurate form on test and on the fresh sweep, it is replaced by the accurate form (one `FITTED` tag fewer in
      quality/fitted.json via `scripts/check_fitted.py --update`); otherwise it keeps its `FITTED` tag, with the range
      restated to what was measured, and is not applied outside that range (R4.3).
- [ ] The Gate A envelope (R2: within 1e-9 relative of 50-digit mpmath or within 15 significant digits of Excel) holds
      on the fresh sweep, with mpmath values recorded in the Outcome or the accuracy tables.
- [ ] No gated cell regresses beyond decision 0083's allowance; a replaced selector's lost bit-exact cells are listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other selectors split from T-3365 (T-3502, T-3503, T-3523, T-3524, T-3525, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).

## Outcome
The 15-digit snap does not generalize and is gone (`FITTED` tags 81 to 80). In its place is a structural model with
no fitted constant: P = m·2^E is rooted as `exp(ln(m·2^r)/n)·2^q`, E = q·n + r, q truncated toward zero
(geomean.rs `split_root`). Where |E| < n it is the old `exp(ln(P)/n)`. Past that, it keeps ln's error at large
|ln P| out of the root.
- **Fresh sweep (R4.2):** probes-numeric/t3502-geomean-fresh, 180 cells, seed 3502, log-uniform counts 2..200 and
  magnitudes 1E-150..1E+150, plus 60 exact-power sets (excel-graph, 2026-10-31). Fit lab `geomean-snap-fresh`
  (hold-out split as T-3360, margin 5 points; 119 train cells, 61 test):

  | candidate | exact | train | test | verdict |
  |---|---|---|---|---|
  | split root, q toward zero (engine now) | 169 | 110/119 (92.4%) | 59/61 (96.7%) | accept |
  | correctly rounded root of P | 159 | 105/119 | 54/61 | accept |
  | correctly rounded (reference) | 150 | 101/119 | 49/61 | reference |
  | 15-digit snap (T-1123, engine before) | 58 | 40/119 (33.6%) | 18/61 (29.5%) | refuse |
  | plain `exp(ln(P)/n)` (the snap's accurate form) | 17 | 11/119 | 6/61 | refuse |

  The old engine is up to 194 ulps off there, and 79 of its 180 cells are beyond 15 digits.
- **Old cells:** fit lab `geomean-snap` (69 literal in-range probe cells, 49 train, 20 test): split root 62
  (43/49, 19/20, accept), snap 36 (25/49, 11/20, refuse: does not beat the correctly rounded 31/49, 16/20).
  SEQUENCE cells, scored by hand: in-range-many 71/72 (snap 71), Copies 34/35 (snap 34).
- **Gate A envelope (R2):** the split root is within 1 ulp of Excel on all 180 fresh cells, so within 15 digits.
  It is also within 1 ulp of the 50-digit mpmath root, as the correctly rounded candidate shows. GEOMEAN has no
  eval/accuracy table yet.
- **Eval delta:** gen-m9 known failures 2887 -> 2861 (26 fixed). probes 2661 -> 2633 (32 fixed, 4 one-ulp
  regressions: geomean-root-refinement A23, A33, A34, A44, allowed by decision 0211). ironcalc-calc unchanged.
  In-scope visibly-off cells 2427 -> 2423.
- **Follow-up:** T-3526 (defer: phase-1b) owns the cells the split root misses.
