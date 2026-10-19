# T-1941 W-C (W4): K1 two-scale solve, state and controls

## 1. Correction to HANDOFF 9.1 / 9.3 (I1) for K1: no c-dependent plateaus
Positive control (`plat_ctl2.py`, run from `review/`):
- At each AN2 sum crossing, two synthetic runs of 16,384 consecutive doubles at c ± 4e-4.
- Step height = mean(sim - exact rational, 80-bit) right minus left.
- The constant added at that stage was perturbed by δ = 0..7 of its own ulps.

Steps (ulps of G), δ = 0..7:

| stage (coef) | crossing | steps |
|---|---|---|
| N.S1 (N5) | 1.0519 | -0.004 -0.004 -0.003 -0.003 -0.002 -0.003 -0.002 -0.003 |
| N.S3 (N3) | 1.7029 | +0.002 +0.001 +0.003 +0.006 +0.006 +0.008 +0.006 +0.008 |
| N.S4 (N2) | 1.3403 | -0.045 -0.046 -0.045 -0.045 -0.042 -0.044 -0.041 -0.042 |
| N.S6 (N0) | 1.5445 | -0.000 -0.002 +0.005 +0.005 +0.000 -0.003 +0.000 +0.010 |
| D.S2 (D4) | 1.2944 | -0.002 -0.004 -0.002 +0.006 +0.005 +0.004 +0.001 +0.005 |
| D.S4 (D2) | 1.1799 | -0.003 +0.004 -0.006 +0.001 -0.002 +0.001 -0.004 -0.001 |
| D.S6 (D0) | 1.3694 | +0.001 at every δ |

Reading:
- In K1 every sum's product sits 1-2 binades BELOW the sum. The exact P + c lies on P's finer grid, so S's rounding is
  decided by P's low bits, which are effectively random from point to point.
- The mean stage error is about 0 whatever c's low bits are: at most 0.01 ulp of c-dependence, below any usable SE.
- 9.1.1's mechanism (S = P + RN_u(c), a constant error) needs P on S's grid, as in K2/K3. It does not apply to K1.
  I1 is not a route for K1.
- (A 256-point subsample earlier showed spurious ±0.05 swings: noise. Use full runs.)
- Committed data near the crossings is sparse. Where measurable, Excel's steps are ≤0.04 ulp, within 1-2 SE.
  P1b (p131-p154) will measure them to SE about 0.006 as a structure check: N.S4 should step -0.045.

## 2. Positive control for the joint phase solve (critique 9.2.4)
Synthetic truth = AN2 + a random combination of its 3 most function-invisible directions:
- max coefficient move 3e5 ulps, function change 2.0e-17;
- simulated exactly on the real grid (9 anchors: e3 incl. runs to 2, e7, e10);
- AN2 matches the truth on 29.6% of the 2.27M-point grid.

(a) Phase rows (5 lines) + Γ rows. FAILS:
- recovers N0 to ≤9 ulps and D1 to 2-4e3 ulps;
- drives N3..N7 / D5..D7 to 1e7-3.5e8 ulps from the truth;
- exact 25-29%.

(b) Phase rows + measured-function rows (91 bins over 2.27M points, SE 0.013 ulp; target minus the reference's own
simulated rounding mean ρ_b):
- without a prior: max error 1.8e8 to 2.2e7 ulps after 4 iterations, still FAILS;
- with a ridge prior of 5e-9 per ulp (1e6 ulps cost one phase sigma):

| iter | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| max error (ulps) | 4.9e7 | 1.8e6 | 1.1e6 | 2.5e5 | 3.9e6 | 8.4e5 |
| exact vs truth | 37% | 45% | 48% | 51% | 38% | 67% |

  - At iter 3 the D chain is within 1-3e3 ulps and N0..N2 within about 1e3.
  - Beyond that it oscillates (12 iterations: 1.5e5-4e6) along one weak direction: alternating N3..N7 / D3..D7.

Section 3 answers whether exact-match search can finish the recovery from the iter-5 point (66.7%): it cannot.

## 3. Why exact-match search stalls near 75% (synthetic, truth known)
- From the joint solve's best point (66.7%), an exact-match annealer reaches 74.9%. The coefficients do not move
  toward the truth: N6 +8.4e5, N5 -3.3e5 ulps.
- Exact rate on the straight line to the truth: t = 0 / 0.2 / 0.6 / 0.9 / 0.99 / 0.999 / 1 / 1.001 gives
  75 / 71 / 60 / 61 / 69 / 75 / 100 / 75%. It is a needle on a ~75% plateau.
- Basin at the truth:
  - one ulp on N0 / N1 / D1 / D2 costs 39 / 24 / 45 / 55% of matches; one ulp on N6 or N7 costs 0.1%;
  - the 6 weakest soft directions drop to 58-94% at 0.03 tolerance units; directions 8-14 tolerate 0.3-3 units;
  - mean |error| has no gradient beyond about 0.1 units (a 0.4-0.6 ulp plateau).
- The five-line joint solve ends 1-4 units off in every direction, its noise floor: about 100x outside the
  weak-direction basins. A ~75% score therefore says nothing about distance from Excel's coefficients.
- Adding all ~27 stage lines (amplitude ≥ 20, amplitude-weighted) makes it worse: N0 goes to about 700 ulps, the max
  error stays at 1.4-2.1e6, exact 28-33%. The weak-stage lines are dominated by phase-law mismatch, not signal.
- Conclusion: with the current data and phase laws, K1's high-stage bits are not identifiable. The remaining routes
  are information-theoretic:
  (i) far more data at the weak stages' lines, plus validated per-stage laws for stages whose product and sum
      binades differ;
  (ii) the pointwise-interval LP/MILP, using every output's half-ulp interval (positive control first);
  (iii) accept AN2 / fitSA as the K1 reference (76-78% exact, about 89% within 1 ulp on all committed data) and
        document the gap.

## 4. Frozen K1 reference
- **G(y), 1 ≤ y < 2:** s = N7; for i = 6..0: s = fl(s*y); s = fl(s + N[i]). Same for t with D. G = fl(s/t).
  - Every op is stored (fl53 of fl64, x87). Degree (7,7), D0 = -1/16.
- **Reference set: AN2** (`../g_candidate_w4_AN2.txt`).
  - 78.2% exact, 89.3% within 1 ulp on the coordinator's 1.45M formula-filtered set.
  - On W4's 1.88M set (G.txt + y1xx + e3 + e7 + e10 in [1,2)): 72.75% exact, 85.97% within 1 ulp, max error 7 ulps.
  - Error histogram 0..7+ ulps: 1369561 248932 190281 67138 5097 1254 333 6.
- fitSA (annealed from AN2) is indistinguishable on the same set: 72.76% / 85.96% / max 7.
- **Recommendation for the engine if we ship non-exact:** AN2, because the coordinator verified and committed it.
  The two are equivalent in accuracy.

## 5. Route (i) for a future task: per-stage law validation, then hidden-stage lines
**Goal.** Bring the joint solve's error in the 6 weakest soft directions from 1-4 tolerance units down to about 0.05.
Those directions are spanned by the hidden stages: N c2..c6 and D c2..c6, products and sums.

**Step 1: single-stage synthetic, per stage.**
- For each hidden stage, simulate AN2 twice on dense runs: with only that stage rounded, and with the stage made
  exact (64-bit). The difference is that stage's own line.
- Fit the line's local frequency and phase law: sawtooth (M + (y-1)M')/2^e, the triangle S'/2^(eS+1), or something
  new where the sum sits 2 binades above the product, or both cross inside the run.
- Validate each law with a known +k-ulp perturbation of the stage's constant, as `fakeruns.py` did for the triangle
  offset.
- Keep only stages whose law recovers the perturbation.

**Step 2: data where each validated line is strongest.**
- Weight of stage k at y is about ulp(S_k)*y^k/D for N, and times G for D.
- Choose anchors inside the stage's binade-stable intervals (between AN2's crossings, section 1 and HANDOFF 9.2.3),
  as far as possible from crossings of other stages.
- Use epsilon ladders (runs of 16,384 at a + c*2^-j, j = 46..22, both sides), 4-6 anchors per stage.

**Step 3:** re-run the positive control (`posctl2.py`) with validated lines only, then Excel.

**Data volume estimate.**
- With the 5 current lines, per-direction precision is limited by noise: 1-4 units on about 2M points.
- If precision scaled as 1/sqrt(N), 0.05 units would need (3/0.05)^2 ≈ 3600x the data: about 7e9 points, which is
  infeasible (about 2 min per 32k oracle rows).
- So route (i) only works if the hidden-stage lines add new, well-conditioned rows: they constrain the weak
  directions directly, rather than averaging noise down.
- The test is cheap: Step 1 for 2-3 stages, then the control with those lines added. If the weak-direction error
  falls by 10x, scale up. If not, K1's exact bits are out of reach with phase methods.

## 6. Files in this directory
- `posctl2.py`: convergent positive control (phase rows + measured-function rows + ridge prior). Saves pc_iter*.txt
  and pc_truth.txt.
- `needle.py`: the line-to-truth scan, one-ulp sensitivities, per-direction basin widths (exact and mean |err|).
- `plat_ctl2.py`: the plateau-reader control (section 1); run from `review/`.
- `steps.py`: Excel/sim step heights at crossings; run from `review/`.
- `gscore.c`: scorer library (gscore exact count, gabserr mean |err|); 4M-point buffer.
- `maxerr.c`: exact / within-1 / max-ulp / histogram.
- `mkgrows.py`: builds grows.npz (y, Excel, Γ hi/lo) from g_rows.txt, needed by `posctl2.py`.
- Not committed (described in W4's report):
  - `posctl.py`, the original failing control (Γ rows only).
  - The final `jointA.py`. It is `../w4/gline_tools/jointA.py` plus these changes:
    - model() makes y contiguous;
    - CUSTOM_F replaces the Γ rows with function rows;
    - RIDGE adds per-coefficient ridge rows;
    - ALLLINES / AMPMIN / AMPW / XREF hooks.
- Also required: libgeval.so (`../w4/gline_tools/geval.c`) next to jointA.py; e3res.txt in ../; e7e10.pkl (built with
  load_rows from the e7/e10 extract_runs dumps).

## 7. Route (i)/(ii) and the generator hypothesis: all negative (synthetic controls; W4, 2026-10-19 PM)
- **Per-stage law validation** (lawcheck.py: known δ on each stage's y^1 coefficient; rms / fraction of runs with
  |r| < 0.05):
  - Valid: N.tri0 .012/1.00, D.tri1 .010/1.00, N.tri1 .029/.92, D.saw0 .034/.91.
  - Usable with outliers: N.saw6, D.tri2, D.tri3, N.tri2.
  - Fail: N.tri4-6, D.saw4-5, D.tri6. The rest are marginal.
- **Adding validated hidden-stage lines** (N.saw6, D.tri3, N.tri2, robust rejection): worse. Max error 4.5-5.7e6
  ulps, 30-38% exact.
- **Numerical-Jacobian joint solve:** the same failure (3.3-8.7e6 ulps). Every ≥1-ulp coefficient change re-hashes
  the hidden roundings, which jitters each line's phase by a pseudo-random amount that does not depend on K.
- **Interval LP with frozen rounding errors** (the half-ulp bounds need long double):
  - At the truth, slack is 0, but its T = 2 step drops the real exact rate to 39%.
  - From 100 ulps off, T = 10 drifts to 140 ulps and T = 200 to 560.
  - Not a finisher at any distance.
- **Known-offset row check:**
  - Below K = 1e12 the predicted shift is ≤0.03 cycles, so all information sits at K ≥ 1e12.
  - There, residual rms is .02-.10 on the low-stage lines but .25-.37 (uniform-random) on N.saw2/5/6, N.tri4 and
    D.saw4/6.
  - So the solves are at their true bound: dirs 7-14 std 1.7-8 units.
- **Fisher design with empirical σ:**
  - +632 cases bring dirs 7-14 to .30-1.4 units; +1,005 cases to .26-1.2 units (√N).
  - Unwrapping the designed top rungs would need intermediate rungs.
  - Not worth about 17 h of oracle time.
- **Generator** (remez.py, gauge.py: exact (7,7) raw-y minimax of Gamma, D0 = -1/16):
  - [1,2] rel: E = 1.44e-18. round53(R*) is about 1e9 ulps from AN2 per coefficient.
  - Other intervals and weights are 1e12-3e16 ulps away.
  - A common scale is not function-null here: D0 shifts G by about 300-570 ulps for s - 1 = 1.86e-7.
  - Phases pin D0 to about 1.5e-11 absolute, so any normalisation rule must keep D0 = -1/16 to 1.6e-10.
  - No normalisation closes the gap. Excluded by more than 50 units in phase-stiff directions.
  - Excel's rational is one-signed against Gamma (+0.45..+1.24e-16 relative, 8 extrema, about 60x the minimax E).
    That is consistent with a finite-precision generator, which cannot be reproduced from outputs.
- **Side channels** (coordinator q501/q502): GAMMA.DIST, BETA.DIST, CHISQ.DIST and 1/GAMMA expose no unrounded G
  bits.
- **Conclusion:** K1 stays at AN2 until a new observable appears. Tools are in W4's scratchpad: ilp.py, jointN.py,
  posctl3/4.py, lawcheck.py, design*.py, rowcheck2.py, remez.py, gauge.py.
