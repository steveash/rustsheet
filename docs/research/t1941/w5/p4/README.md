# W5 part 4 (W-B): K3 plateau reader, GAMMALN near-1 on [0.7, 1.5) (T-1941)

Plan: HANDOFF section 9.3 I1 and 9.4 W-B. Base: c5_f99t. Data: every committed GAMMALN row in [0.7, 1.47),
excluding |x-1| < 2e-3 and the N.S4 zero region above 1.47. Bias is measured relative to c5's smooth exact
function, in relative ulps (2^-52 of the reference). A smooth reference cancels in every step and is absorbed by a
degree-8 polynomial in the global objective.

## Tools
- `events.py`: binade crossings of every product P_j, sum S_j, A and res of a candidate. My naming:
  S_j = fl(P_j + c_j), so the review's N.S9 is my N.S0.
- `stepreader.py`: the local step reader.
  - At each crossing cluster of one stage, isolated from other stages by W = 0.002, it measures
    target step = (right minus left mean) of (target - candidate sim), using all points.
  - Every trial move c_k + j·ulp(c_k), |j| <= 2048, is simulated exactly on the window points.
  - It keeps the j within 2.5 SE of the target step and intersects the sets over the stage's crossings.
- `plateau.py`: the global staircase chi^2 (per-run means, smooth polynomial removed) and an exhaustive coordinate
  search. `chi2map.py` maps chi^2 by x. `apply.py` applies the readings and restores the function.
  `linreader.py`: the linear segment model (ill-conditioned, see below). `check_alias.py`, `truebeta.py`: controls.

## 1. Positive controls
Synthetic truth: c5 with N0 +37, D1 -11 and D2 +3 ulps, on the real x grid.
- **Step reader.** The truth j is consistent at every crossing (30 of 30). D2, the only flip that changes a stored
  sum, is the only stage where c5 (j = 0) is excluded. N0 and D1 correctly stay ambiguous: their flips keep
  RN_u(P + c) in every configuration, so the computed output does not change. `truebeta.py` confirms that their
  stage-error change is exactly -delta/u, which cancels the function change.
- **Global coordinate search from c5** sticks on a joint alias, chi^2 370 against 0 for the truth (all points: 4982
  against 0). Use it only after the local reader, from a nearby start.
- **Linear segment model.** It explains the synthetic staircase (chi^2 26204 -> 125), but its 49 segment
  regressors are collinear with the polynomial: D.S2 seg2 reads +0.94 (true ≈ +1.0), seg3 +14.5 (true ≈ 0). Not
  used for readings.

## 2. Excel's step readings (`stepreader_real.txt`)
- Step heights (relative ulps): N.S0/P0 at 1.0678 +1.02; N.S1 at 0.7303 -0.55; N.P1 at 0.7513 +0.48;
  D.P2 at 0.8619 -0.44; D.S1 at 0.8783 -0.51; D.P1 at 0.8806 +0.50; D.S1/P1 at 1.1067 +0.48; N.S1 at 1.1168
  +0.52; D.S3 at 1.3422 -0.25; D.S1/P1 at 1.3703 +0.51.
- **c5 is excluded for N0, N1, D1, D2 and D3.** Every other stage's isolated steps are consistent with c5.
- Nearest consistent moves (ulps of each constant):

  | stage | nearest consistent j |
  |---|---|
  | N0 | +436..+443 |
  | N1 | -18..-24 |
  | D1 | +306..+313 |
  | D2 | +3..+9 |
  | D3 | odd j (±1, ±3, ...) |

- **Cross-check with the tie readings.** D3 odd moves are consistent, and j = -1 gives ...b68, a multiple of 4 ulps
  (W5 part 2) ✓. N2 has no isolated step reading here; the tie reading (even) stands.

## 3. Applying the readings
- `apply.py`: c5 + {N0 +440, N1 -21, D1 +310, D2 +6, D3 -1, N2 +1}, then integer moves of N3-N9 and D4-D9 to
  restore c5's function (max relative difference 3.8e-17).
- With that set as base, the step reader finds every isolated crossing consistent: j = 0 lies in every stage's set
  (`stepreader_real_it2.txt`).
- Exhaustive coordinate search on the staircase objective from there (64 points per run, then checked on all):
  further moves N0 +252, N1 +12, N2 +1, N5 -2, N7 -6, N8 +5, D2 +1, D7 +5, D8 -2. Saved in `plateau_real.txt`.

| set | staircase chi^2, all points (499 runs) | exact far | exact near |
|---|---|---|---|
| c5_f99t | 35,400 | 0.318 | 0.746 |
| c5 + bits | 29,972 | 0.150 | 0.164 |
| + function restored (`k3_bits_restored.txt`) | 7,859 | 0.246 | 0.491 |
| + coordinate search (`plateau_real.txt`) | **2,438** | 0.266 | 0.463 |

- The bit readings remove 93% of the staircase chi^2. The rest is concentrated at 0.70-0.78, 0.98-1.02 and
  1.06-1.14 (`chi2map.py`).
- The exact score is lower, as the review predicts (9.1.3). The function restoration moved N3-N9 and D4-D9 freely
  and so broke c5's near-1 stage structure. Exact match is the final verifier, not the objective.

## 4. Not done (budget)
1. Step (4): refit the function-visible directions to Excel's staircase-corrected curve (I4) while keeping the
   near-1 stage Taylor structure (`../p3/pinfit.py`, loose order-0 pins). Then decide the degree by chi^2.
2. Readings at the non-isolated clusters: 0.8202-0.8224 (N.S0/P0 with D.S2), 1.1618-1.1623 (D.S2 with D.P5),
   1.3033-1.3046 (N.P4 with D.S5), and the D.S0 crossings, which read D0's low bits.
3. Step (5): score.
