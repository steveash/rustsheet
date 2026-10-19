# W5 part 5: product-level phases on top of ss_fc (T-1941, K3)

Base: `../p4/ss_fc.txt` (plateau bits read, function corrected).
Synthetic controls use function-preserving moves: random combinations of the softest SVD directions of the
relative-function map, about 1e5 ulps, with function change of about 4e-17.

## Control-backed negatives
1. **Per-site phase meters fail when every stage differs** (`validate.py` with `inj.c`, `validate2.py` with
   `meter.py`).
   - The meters: injected-shift matched filter (discrete output flips), and a continuous template (the site's own
     rounding-error sequence at phase phi, times its propagation weight).
   - On a function-preserving synthetic move, per-run phases are random for every site (rms 0.2-0.47 cycles), at
     1,024, 4,096 and 8,192 points per run.
   - Cause (`dbg2.py`): a site's Excel error sequence is a phase-shifted copy of the candidate's only where its
     upstream product rounds the same way. For N.S7 the match is 65% at the true phase, because the upstream
     ulp(P7) = u(S7)/4 scrambles the rest.
   - The top sites (N.S7, N.S6, D.S6, D.S5) are also weak: template norm 0.3-2.7 against a residual norm of about
     45 per 8,192 points, so per-run SNR is about 1.
2. **No partial credit along the sloppy directions** (`lambda_path.py`). With truth = base + m, the set
   base + lambda·m scores exact 0.26-0.33 and residual correlation <= 0.17 for every lambda up to 0.9999 (about 4 ulps
   off on the mid coefficients). It scores 1.000 at lambda = 0.99999. The exact-match basin is about ±1-2 ulps on
   the mid coefficients. No whole-output objective (exact rate, correlation) guides a search there.
3. Bug found and fixed on the way: the residual is (B - ref)/|ref|, and res = t·A changes sign at x = 1, so every
   template or correction needs a sign(res) factor (`meter.py`, `../p4/fcorrect.py`).

## What does carry signal: coherent multi-run sums at the true phases (`coherent.py`)
Z of the sum over runs of the correlation at the *true* per-run phases, against random phase assignments, on the
synthetic:

| runs | N.S7 | N.P6 | N.P5 | D.S6 | D.P5 | D.P4 |
|---|---|---|---|---|---|---|
| 60 runs of 1,024 points | +3.0 | +1.5 | +5.1 | +2.3 | +3.7 | +6.3 |
| 43 runs of 4,096 points | +2.8 | +4.1 | +6.1 | +3.6 | +4.4 | +7.6 |

Taking each run's best phase gives Z of 8-14 at random, so per-run phases cannot be used. A global fit of each
site's smooth phase function, ΔS_j(x)/u, a polynomial with a few unknowns, is statistically feasible with all
real runs (about 700, so Z about 15-20 expected). Plan: a top-down global scan, N.S7 first, then one coefficient
per lower product site, FFT-accelerated.

## Top-down pipeline: end-to-end control on the N chain (current x grid, 623 runs)
Truth: ss_fc + a 1e5-ulp function-preserving move (`synth.py`, seed 1; `e2e_truth.txt`). Base: ss_fc, so every
coefficient differs.

1. **N top (N7, N8, N9), `topgrid2.py`.**
   - Step 1: an (N8, N9) grid on the exact S-only objective (`hypo.c`, sites S8, P7, S7, without P6). It does not
     depend on N7's whole-u shifts.
   - Step 2: N7's integer by an FFT scan on P6 (`mscan.py`; harmonic sawtooth expansion, df = 1/NF so the FFT index
     equals m).
   - Step 3: an exact 3-D refine.
   - Start: the scan (`scan2.py`) applied (`apply_top.py`). Result: **N7 exact, N8 +2 ulps, N9 -389 ulps** (inside
     the basin: about ±3 ulps for N8, ±500 for N9). Objective 299 against the truth's 298.
2. **N6..N1, `fscan.py`.** An FFT peel over k = R·q + r: S_j exact per residue r, then the product P_{j-1} as a
   sawtooth in q with per-point frequency. k range ±2e6, 0.5-2 min per coefficient (14 min for N0, R = 2048).

   | coefficient | N6 | N5 | N4 | N3 | N2 | N1 |
   |---|---|---|---|---|---|---|
   | truth rank | 0 | 0 | 0 | 0 | 0 | plateau |
   | Z (next) | 6.4 (5.3) | 12.0 (10.5) | 6.0 (4.7) | 28.2 (26.2) | 10.5 (8.4) | 14.3 |

   N1's plateau is 5 consecutive k with equal value; they give the same stored values (one rounding cell).
   - Lesson: **do not refit the function between peels.** A least-squares refit after N6 moved N5 by 1.5e8 ulps,
     out of every scan range (`e2e_c*.txt`). Peeling needs only the upstream coefficients.
   - **N0 is not peelable from the N chain.** Its next rounding is the division A = N/D, so it comes after D.
3. Earlier exact two-site template (`peel2.py`, `peelk.c`): rank 0 for N3, N6, D3 within ±2e4, at 1.5 ms per
   hypothesis. The phase-only model (`peel.py`) fails where ulp(c_j) < u(S_j), because ties scramble the product.

## Range problem for real data
The function does not bound the top tails. With the lower coefficients compensating, the principal ranges of
ΔS_top, in cycles of the stage's ulp, are:

| tail | at ε = 1e-17 | at ε = 1e-18 |
|---|---|---|
| N.S7 (quadratic) | 4.4e3, 6.3e5, 1.4e8 | 436, 6.3e4, 1.4e7 |
| D.S6 (cubic) | 72, 2.9e3, 1.4e5, 1.8e7 | 7, 294, 1.4e4, 1.8e6 |

So a box like ±3e4 (linear) × ±200 (quadratic) may not contain Excel. The real N.S7 scan on the current data
peaks at the box edge (b 29,890) and is inconclusive (`scan_real_NS7.log`).
Plan:
- Near x = 1, take (a, b') from the near-1 runs, where the curvature is negligible.
- Then take c by a 1-D FFT over the far runs.
- Same for D.S6.
- Plus the new n1001-n1128 data (about 2.1M points), which roughly doubles the coherent Z.

## Control chain closed (N0 last)
`n0scan.py` (exact-match rate over N0 with all other coefficients fixed, coarse step 256 ulps then
fine) on `e2e_f1.txt` gives `e2e_final.txt`: **0.9997 exact vs the synthetic truth** (ss_fc start
0.3066). N0 lands −66 ulps from truth inside its exact-equivalent plateau, so the pipeline recovers a
bit-exact-equivalent set on the control. Residual differences from truth are all on tie plateaus
(N1 +3, N8 +2, N9 −389, D1 +25), with no effect on outputs. Open: the D top (D6..D9), which the control
took from the truth base. scan3 (cubic D.S6) is running.

## D top, and the control chain from scratch
- `scan3.py` (cubic D.S6, CMAX 50, DMAX 2) finds b −1591.0, c −19.5 (truth −1590.8, −19.0); its cubic term is
  unresolved, so `dtop.py` converts with d = 0 → D7 +8, D8 −275, D9 +1e5 ulps from truth.
- `topgrid2.py` with CHAIN=D (J0=7 objective): D8 within 2 ulps, but D7/D9 wrong. The found set outscores the
  truth there (152 vs 147), so J0=7 is too weak for the D top.
- `dtop3.py`: (D7, D8, D9) grid on the J0=6 objective (P5 excluded). Result: D7, D8 exact; D9 −9824 ulps on a tie
  plateau (713 vs truth 712; coarse Z 5.7).
- `e2e_hchain.sh`: peeled N side plus dtop3 D top, then peel D6..D1, then `n0scan.py`. Truth at exact rank 0 for
  every D step (D6 Z 6.8, D5..D2 Z 1.4–4.2, D1 tie plateau); final **0.9997 exact vs the synthetic truth**
  (`e2e_hfinal.txt`).
- The whole pipeline runs from ss_fc on the control with no truth input: N top scan → topgrid2 → N peel →
  scan3/dtop3 → D peel → N0.

## Real data: D top scan (scan3, rows4 incl. V3, 807 runs, c ±200 step 0.5, d = 0, b ±1e5)
No detection. The best is 316.3 at b = 0, c = 0, a = 0.53, i.e. the base shape shifted by a half-ulp phase, which
flips the odd harmonics of the base S6 pattern. Null mean −5.5, sd 67.3; the best of ~8k kept candidates is at
the expected null maximum. (Control, same settings: the truth was the top candidate at 257 with null sd 58.7.)
Structure check (`structtest.py`, 300k real points, ss_fc): drop the fl53 store at one stage at a time. No stage
gives a sharp gain (best D.S3 0.3624 vs base 0.3550; every top-stage drop is neutral within ±0.0002; the A store is
required, 0.2898). So the half phase is not a missing store. Reading: the real D top lies outside the scanned
box. The function allows D.S6 principal ranges of 72 / 2.9e3 / 1.4e5 / 1.8e7 cycles in (a, b, c, d), and the
control truth was planted inside a small box.

## Range problem, quantified on the control
- The function barely bounds the top tail. `make_truth` moves of 1e7–1e9 ulps on the top coefficients (soft SVD
  directions, kmin 8–15) change the function by only 2e-17 to 1e-16, the rounding floor. The real top can be that
  far from ss_fc.
- The S8 stage (linear, the only scan with no grid) carries no signal. Control truth scores: N.S8 8.4 (TSIZE 1e5) /
  21.3 (1e9), null sd ~35; D.S8 −19.7 / 23.1, null sd 28. So a top-down scan has to start at N.S7 (quadratic) and
  D.S6 (cubic), which need c (and d) grids.
- Planted N.S7 truths: TSIZE 1e6 → (b, c) = (−5537, 60.9), S 260; TSIZE 1e7 → (−55366, 608.5), S 280, joint null
  sd 48. Range control `launch_scan_farctl.sh`: same box as the real wide scan (b ±1e5, c ±2000), truth at TSIZE 1e7.

## Where the top-stage information lives (`perrun.py`, PERRUN dump of scan2/scan3; control TSIZE 1e7, rows4)
The truth contribution per run is compared with the null spread over phase (`perrun_tsize1e7.log`; the cubic term
is now included in scan3's at-truth score):
- N.S7: total Z 10.3. Concentrated at x ≥ 1.2: Z²/run 0.33–0.51, against 0.04–0.07 for x in 0.8–1.2 and ~0 below
  0.8.
- D.S6: total Z 11.2, broad (Z²/run 0.05–0.25 everywhere).
- N.S6: total Z 8.6, patchy. Not better than N.S7 here (W6's (1.5,2] kernel differs).
So new K3 data for the top scans should go to x in [1.2, 1.47) for N, spread uniformly for D.
