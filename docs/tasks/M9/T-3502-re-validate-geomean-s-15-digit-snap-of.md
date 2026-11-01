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
- **First fresh sweep (R4.2 for the snap; it also chose the replacement):** probes-numeric/t3502-geomean-fresh, 180 cells, seed 3502, log-uniform counts 2..200 and
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
- **Model choice:** made on the T-1123 probes and the train side of the first sweep only. q toward zero beat q
  floored there: in-range-many 71 against 53, and train 110 against 104.
- **Post-freeze sweep (R4.2 for the split root):** probes-numeric/t3502-geomean-postfreeze. It uses the same
  strata with a new seed (35020) and was generated and filled after `split_root` was frozen (excel-graph,
  2026-10-31). Fit lab `geomean-postfreeze`, 124 train and 56 test cells:

  | candidate | exact | <=4 ulp | <=15 digits | train | test |
  |---|---|---|---|---|---|
  | split root (engine) | 172 | 180 | 180 | 118/124 (95.2%) | 54/56 (96.4%) |
  | correctly rounded (reference) | 165 | 180 | 180 | 110/124 (88.7%) | 55/56 (98.2%) |
  | correctly rounded root of P | 165 | 180 | 180 | 111/124 | 54/56 |
  | 15-digit snap (T-1123) | 70 | 98 | 115 | 45/124 | 25/56 |
  | plain `exp(ln(P)/n)` | 25 | 84 | 115 | 17/124 | 8/56 |

  The R4.1 hold-out that decides is the first sweep's, where the model was chosen on the train side only: test
  59/61 against the reference's 49/61, so it is accepted. This post-freeze sweep is the R4.2 record. The split
  root leads overall (172 against 165) and on train, and is one cell behind the reference on this sweep's 56 test
  cells, where the per-sweep rule reads "refuse". As a supplementary view only, since it includes cells used to
  choose the model, fit lab `geomean-all` pools the T-1123 probes and both sweeps: train 271/292, test 132/137,
  against the reference's 242/292 and 120/137. The correctly rounded mean cannot be the engine's form anyway: it
  breaks gated ironcalc-calc GEOMEAN C23 and G27.
- **Gate A envelope (R2):** holds on both sweeps. Every split-root cell is within 1 ulp of Excel, and within
  1.3E-16 relative of the 50-digit mpmath root of P (the bound is 1E-9). Per miss, Excel / engine / mpmath:
  - post-freeze A5: 1.1257931344394546E-52 / ...544E-52 / 1.1257931344394544661E-52
  - post-freeze A79: 6.130873813628597E-07 / ...598E-07 / 6.130873813628597051E-7
  - post-freeze A99: 1.4231555564608051E+32 / ...053E+32 / 1.4231555564608051535E+32
  - post-freeze A115: 1939982650394.7751 / ...775 / 1939982650394.7750115
  - post-freeze A47, A50, A70, A74 and the 11 first-sweep misses are likewise below 1.2E-16 relative.

  GEOMEAN has no eval/accuracy table yet.
- **Eval delta:** gen-m9 known failures 2888 -> 2862 (26 fixed: 25 GEOMEAN cells and a PPMT over GEOMEAN). probes 2661 -> 2633 (32 fixed, 4 one-ulp
  regressions: geomean-root-refinement A23, A33, A34, A44, allowed by decision 0218). ironcalc-calc unchanged.
  In-scope visibly-off cells 2427 -> 2424.
- **Follow-up:** T-3526 (defer: phase-1b) owns the cells the split root misses.
