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

## Unstored-op fingerprint on K3 (after W2's K2 finding: N s4, D s3 unstored)
`oprep.py` (400k real points, ss_fc constants): drop the fl53 store at one op and count repaired minus broken rows.
D.S3 is clearly first (repaired 12999, broken 9986, net +0.75%), then D.P3 +0.38%, N.P5 +0.23%, N.S5 +0.17%. Every
other op is neutral or negative (low stages −0.2% to −2.5%). Pairs: D.S3+N.P5 +0.98%, D.S3+N.S5 +0.93%;
D.S3+D.P3 cancels (+0.03%). `oprep_real.log`.
`structclimb.py` (greedy single-coefficient climb from ss_fc, 200k points): all-stored 0.3542 → 0.3591; D.S3
unstored 0.3621 → 0.3655; D.S3+N.P5 0.3643 → 0.3644. The climbs stall at once (sloppy directions), so this is
suggestive only. `oprep_ctl.py` (synthetic truth e2e_truth under a structure, screened with ss_fc) is not a valid
control: the lower-coefficient mismatch between e2e_truth and ss_fc makes every low op "repair". A real control
needs constants fitted under the wrong structure (W2's method).

## Per-point stage reads (W2's K2 route) on K3: not informative
`stageread.py`: with the late chain from ss_fc, a per-point shift of ±3 ulps at the stored N.S5 and D.S3 explains
91.6% of 200k real points (±8: 96.0%). But B shifted by 1 ulp, which is wrong by construction, is also explained at
~70%, with a median of 7 of 49 solutions per point. The reads are too ambiguous to sample the stage functions.
`runread.py`: within a run a smooth stage error would be constant, but no run-constant (dn, dd) explains the runs.
The peak hit rate per run has median 0.30 (2nd best 0.297, null 0.27). So K3's mismatch at these stages varies from
point to point (rounding-pattern type), not a smooth stage offset. W2's correction applies: the oprep hot spots
(D.S3, N.P5/N.S5) localise the mismatch; they do not show an unstored op. `stageread.log`, `runread.log`.

## Hand-off model test (W2's K2 model) on K3: not diagnostic
`handoff.py`: the stored N.S_JN and D.S_JD are fl(smooth upper polynomial + δ), δ constant per run on a u/4 grid
over ±3 ulps, with the exact stored chain below. Per-run best hit rates (`handoff.log`, 12 runs × 1024 points):
- Real, ss_fc constants: 0.25–0.49 at N.S5/D.S3, 0.31–0.52 at N.S2/D.S1, 0.30–0.54 at N.S1/D.S1. Plain Horner
  scores 0.11–0.44 and the null (B + 1 ulp) 0.17–0.49.
- Control, plain-Horner truth scored with its own true constants: 0.56–0.92 at N.S5/D.S3 and only 0.39–0.58 at S1.
On K3 the upper Horner's point-to-point rounding noise stays visible down to the output, so even the true model is
not explained by a per-run δ. Real data at S1 is about the same as this control. The test cannot separate plain
Horner from a smooth (higher-precision) upper part on K3, unlike K2 where the upper is invisible.

## Restartable scans
A container restart killed the real wide N.S7 scan (~7.5 h in) and the first far-truth control. scan2.py now takes
CPART=i/n (scan every n-th c; parallel parts), CKPT (per-c candidate checkpoint; a rerun resumes and skips the done
c values) and MERGE=1 with CKPTS=f0,f1,... (no scanning; jointly rescore the union of the parts' candidates).
`launch_farctl_parts.sh` runs the far-truth control as 4 resumable parts (~2.2 h).

## Sub-ulp offset meter at N.S5/D.S3 (HANDOFF §12.3 step 1, adapted): no lift
`deltameter.py`: stored chain, with an offset δ (u/16 grid, |δ| ≤ u/2) added to the pre-rounding sums at N.S5 and
D.S3; per run, the best (δN, δD). 12 runs × 2048 points: mean exact 0.346 at δ = 0, 0.365 at the per-run best. No
run goes beyond a few points (`deltameter.log`). On K2 the late chain was right (99.97% per anchor) and one hand-off
offset explained the rest. On K3, ss_fc is ~35% exact and the mismatch is spread over many stages, so the
N.S5/D.S3 hot spots are coefficient symptoms (review 3 §12.4), not a single fixable site.

## Is K3's D0 a power of two? (after W6: Excel's D0 is not 1 on (1.5,2])
The K3 tools never normalise by D0; they hold D0 = 2^-13, which W1 fixed in its fit (q0 = 2^-13), never measured.
- `d0scan.py` (D0 alone, everything else fixed, 100k real points): exact rate flat at 0.3555 for |k| ≲ 600 ulps
  (±1.3e-13 relative), falling outside (0.30 at ±4096, 0 beyond ±2.6e5). D0 alone is pinned to 2^-13 within 1.3e-13.
- `cscan.py` (exact common scaling (N, D) → (fl(cN), fl(cD)), c = 1 + k·2^-52, |k| ≤ 8192, i.e. ±1.8e-12): the
  function is unchanged and only the roundings move. c = 1 is best (0.3555; median 0.269, sd 0.036; next 0.3532 at
  k = +1618). There is no peak above it. ss_fc was fitted at c = 1, so a peak there is built in; a true c* ≠ 1 would
  have to beat it, and none does in this range. A far c* is not excluded. `d0scan.log`, `cscan.log`.

## Far-truth range control: detected (S-only)
`launch_farctl_parts.sh` (4 parts, merged with MERGE=1; `scan_farctl_NS7_merged.log`). Planted truth at TSIZE 1e7:
N.S7 (b, c) = (−55365.7, 608.47), wide box b ±1e5, c ±2000 (8001 c values × 64 phases × ~2e5 b, ~1.6e9 cells),
rows3, 751 runs.
- S-only: the truth is rank 0 (274.1 at a 0.48, b −55365.6, c 608.5; ranks 1–2 are its neighbours). The best
  unrelated candidate is 232.6, an alias at the true c with wrong b (17285.5). Kept-candidate median 157, sd 6.8.
- Joint (S + P6): worse. A base-like candidate (b ≈ 0) comes first, and the truth's P term is −46.6. For K3, use
  S-only to rank.
So a function-unconstrained top tail 1e7 ulps away is found by this box scan on current data.

## Data hygiene: q501/q502 have B = GAMMA, not GAMMALN
The shared extractor (`t1941/extract_runs.py`) assumes column B is GAMMALN.PRECISE for every kmyst-gam case.
kmyst-gam-q501 (x0 1.3171, inside K3) and q502 (x0 ≈ 1.77) have B = GAMMA. rows3/rows4/rows5 contain q501 (one run of
8192 points with residuals ~1e16 units); rows2 (ss_fc's fit) does not.
- Affected: the real D.S6 scan (rows4) and the killed real N.S7 wide scan (rows3). The controls synthesise B and
  are unaffected; the exact-rate screens lose only 8192 points.
- `mkrows.py` builds the clean set (B formula checked): rows6, 5.68M points, 369 cases incl. 112 k3t.
- Real wide N.S7 scan relaunched on rows6 (918 runs), 4 resumable parts (`launch_real_parts.sh`).

## Real wide N.S7 scan on clean rows6 (918 runs): no far top inside the box
S-only, merged over 4 parts (`scan_real_NS7_merged_Sonly.log`): the top is the base shape (b = 0, c ≈ −1..0) at phase
a ≈ 0.5–0.6 (352.1). The best candidate away from the base is 317.8 (b −11.0, c 20.5). Nothing distinct like the far
control's truth (rank 0 there, 41 above the best unrelated candidate).

What "base shape at a ≈ 0.5" means (`APROFILE=1` in scan2/scan3: S at b = c = 0 as a function of a). The residual is
Excel − (stored base model). If Excel's rounding at a site is unrelated to the base's (far-off coefficients, or an
unstored site), the residual carries −w·e0, the base's own rounding at that site, removed. Then
C(a) ≈ −ρ(a): strongly negative at a = 0, peak at a = 0.5, symmetric about 0.5.
- Real N.S7: S(0) = −728 (Z −17.5); S(0.5) = 352 (Z 8.6); S(0.375) 272 vs S(0.625) 290, S(0.125) −203 vs
  S(0.875) −261. Nearly symmetric, so this is not a specific hidden phase offset; Excel lacks the base's N.S7 pattern.
- Real N.S3: S(0) Z −28.4. Real N.S1: Z −2.3. Real D.S2: Z −2.8. The read sites (N1, D2) mostly match; N.S3 does
  not (N3 is fitted, not read).
- Control (truth = ss_fc = base) at N.S7: S(0) Z −1.5 (≈ 0, as it should be).
So the D.S6 "a = 0.53" result and this one say the same thing: the base's top-site rounding is absent from Excel,
either because the true top is outside the box or because the top is unstored (W4's K1 finding). The data cannot
separate these yet. Per-site map running (`sitemap.sh`).

## Hidden-term test (the coordinator's K2 lead): no effect on K3
`hidden.py`: S_j = fl(P_j + C_j + c/x) (extended, one store), C_{j−1} − c at the next sum; an algebraic no-op that
moves only roundings. 50k clean points, every N and D stage, c in ulp(S_{j−1}) units, ±10,000 step 4
(`hidden_ssfc.log`), and at the top stages c in ulp(S_j)/32 over ±4 ulps (`hidden_fine_top.log`). Best gains +0.7%
(D.S2 0.3393, D.S4 0.3389 vs 0.3320) and +0.85% (N.S6 fine). No K2-like jump (K2: 0.74 → 0.88).

## Per-site anti-base map (real rows6, ss_fc base; `sitemap.sh` → `sitemap_N.log`, `sitemap_D.log`)
Z of the S objective at a = 0 against random-hypothesis nulls. Strongly negative means the base's rounding pattern at
that site is missing from Excel (wrong phase, wrong coefficients or unstored); near 0 at a strong site means present.
- N: S8 −0.7 (no signal), S7 −17.5, S6 −28.0, S5 −8.9, S4 −16.1, S3 −28.4, S2 −2.9, S1 −2.3.
- D: S8 −0.3 (no signal), S7 −4.0, S6 −18.0, S5 −28.4, S4 −95.2, S3 −66.8, S2 −2.8, S1 +1.9.
N.S1, N.S2 and D.S2 are strong sites (null sd ~300) whose base pattern IS present. Excel stores them, and ss_fc's
read constants there are right. Everything from N.S3 and D.S3 up is mismatched; D.S3/D.S4 are the worst. This
argues against an all-80-bit D chain and an unstored N.S2. A no-refit mask gain is not evidence either
(`maskctl.py`, `maskctl.log`): on a fully stored synthetic truth, ss_fc with D unstored and N stored only below 1
gains +8.6% (real: N<2 with D unstored, +5.7%).

## Structure controls for the K3 refit, and a failed quick check
- `maskmodel.py`: the masked model ("stored below k": the ops at coefficient index j are rounded iff j < k).
- `mkctl.py`: planted controls (ss_fc + a function-preserving move, TSIZE 1e5), with B synthesised under a mask:
  `ctl_m3.txt` (stored below 3 on both chains), `ctl_m2.txt` (W2's mask: N below 2, D fully 80-bit) and
  `ctl_st.txt` (all stored). Handed to W2 for the integer refit.
- Quick check (`lsmask.py`, `lsmask.log`): a Gauss–Newton LS refit of N3..N9 and D4..D9 under each mask, then the
  exact rate. It FAILS its controls. On the all-stored control the m2 mask wins after refit (0.367 vs 0.308 for the
  true stored structure), and m2 also wins on ctl_m3. The head runs off along function-null directions (D9 by
  ~7e15 ulps). Masked models score higher whatever the truth, because they carry fewer mis-phased roundings. Real
  data ranks m2 0.382 > m3 0.354 > stored 0.309, which by the controls means nothing. Only an integer refit that
  closes on its own control can decide.
- W2's function-free structure test (W2's k3/lsmask.py): from Excel's y, take the target A* (doubles with
  fl(t·A) = y), weighted-LS fit N − A*·D, residual rms in ulp(A); each structure is simulated with the fitted
  function as calibration. Real 1.013 / median 0.663; all-stored simulation 1.018 / 0.665 (match); W2's mask
  0.579 / 0.375; partial masks 0.72–0.93. **K3 is fully stored.** The mask lift was ss_fc's coefficient errors being
  hidden by unstored ops. So the site map's mismatch from N.S3/D.S3 up is wrong mid and top coefficients.

## Mid-coefficient near-null lattice from ss_fc (K3 is stored: W2's fingerprint and `../fp/`)
- `k3lat.py`: BKZ on [I | λJ] over N3..N9, D3..D9 (N0..N2, D1, D2, D0 fixed). Eleven mid vectors of 1–4 ulps with a
  function change of 0.002–0.022 ulp(A) (`k3lat_100.log`).
- `k3climb.py`: exact-count climb (±1, ±2 of each vector and all ±pairs; 300k train / 300k held-out on rows6).
  0.3320 → 0.3391 train, 0.3332 → 0.3405 held-out, then a local maximum after 10 steps. Total move N5 −2 N6 +1 N7 −1
  N8 +9 N9 −3 D6 +1 D7 −4 D8 −7 (`k3climb.log`, `k3lat_climb.txt`).
- `k3cvp.py`: integer move (CVP) taking ss_fc's function toward the real-data LS fit (`../fp/` fit, residual 1.01
  ulp(A)). The ss_fc → LS gap is max 2.6, rms 0.99 ulp(A). The CVP moves cut it to rms 0.54 / 0.47 / 0.38 (λ = 30 /
  300 / 3000, moves up to ~1100 ulps), but held-out exact DROPS: 0.333 → 0.295 / 0.291 / 0.284. So the LS-fit
  function is not the stored chain's function target (the LS fit absorbs the stored chain's rounding bias), or the
  fixed read constants are part of the gap.
So ss_fc is near a local maximum in the near-null lattice, as AN2 is on K1 (W4). The truth is far along null
directions; local search from ss_fc does not reach it.

## Per-stage phase read on K3 (review 3's phase method, generalised: `k3phase.py`, `k3phase_an.py`)
Per chunk of ≤ 8192 consecutive doubles (594 chunks from 376 rows6 runs), each stored sum S_j leaves its rounding
sawtooth as a spectral line in (y − exact)/ulp(y) at f_j = S_j'(x)·ulp(x)/ulp(S_j). The phase of Excel's line relative
to the stored ss_fc model's, signed by a planted +0.25 u offset, is the pre-rounding offset δ_j mod 1 u
(`k3phase_ssfc.npy`: x, then per stage phase, amplitude ratio, calibration shift, line power, f).
- Readable stages (|calib| > 0.12, amplitude ratio 0.5–2, power > 4): N7 240 chunks, N6 412, N5 96, N3 518, D6 243,
  D5 426, D4 91 (D3 22, D2 32). N8, N4, N2, N1, D8, D7, D1 have no usable line.
- Neighbouring chunks agree better than random (adjacent |Δphase| vs random): N3 0.138 vs 0.251, N6 0.157 vs
  0.252, D5 0.146 vs 0.251, N7 0.185 vs 0.247. So δ_j(x) is a smooth law for these stages.
- But no simple global law fits (x, 1/x, x², x³ with |B| ≤ 400: resultant R ≤ 0.31, `k3phase_an.log`). Between chunks
  ~0.01 apart the phase moves 0.3–0.5 cycles, so the laws have slopes of order 10²–10⁵ cycles per unit x. The
  current sampling cannot unwrap that: runs ~0.002 apart, chunks within a run ~1e-12 apart, and the V3 rungs
  (2^-24..2^-46) resolve slopes ≳ 1e7.
- Data that would unwrap them: ladders near a few anchors (e.g. 1.25, 1.35, 1.45, 0.8, 0.95, 1.1), each with ~12 runs
  of 8192 consecutive doubles at offsets Δx = 2^-10 .. 2^-21 from the anchor. That is 72 runs, 0.6M points. The
  local slope of δ_j at each anchor then comes out by stepwise unwrapping, giving one far null coordinate per
  readable stage.
- Ladder probes for the unwrap are committed: kmyst-gam-k3l[a-f](00|10..21), 78 runs × 8192 (`../gen_k3ladder.py`),
  queued for the oracle.
- Interim slope read from existing close pairs (`k3slope.py`, `k3slope.log`; rows7 with 208 k3t, 786 chunks). Per
  0.05-wide window, the slope s maximising Σcos 2π(Δφ − sΔx) over pairs with 1e-5 < Δx < 1e-2, |s| ≤ 3e4: peaks at
  Z 4–5 with second peaks almost as high, and the sign jumps between windows. That is the noise maximum for ~1.2e5
  slope values: not resolved. Pairs closer than 1e-6 (the V3 rungs) agree in phase at N3 (0.130 vs random 0.237), so the
  reads are real and |slope| ≪ 1e6 there. The unwrap needs the ladders.

## Ladder unwrap, built and validated on controls before the data lands (`phaselib.py`, `ladder.py`)
`ladder.py`, per anchor and per readable stage:
- read the line phase of the base run and of each rung (anchor + 2^-kk), relative to the base;
- unwrap from kk = 21 up to 10, giving the local slope s_j(anchor) of the pre-rounding offset (cycles of ulp(S_j) per
  unit x);
- jointly solve the slopes for the coefficient moves (s_j·ulp(S_j) = Σ_{k>j} ΔC_k (k−j) a^(k−j−1)), with iterative
  outlier rejection (residual > 300 cycles/x).
Controls with B synthesised at the exact ladder x from planted far truths (`ladder_ctl_1e7.log`,
`ladder_ctl_seeds.log`):
- Single slopes with ≥ 10 usable rungs match the truth to < 0.2% (N3 at every anchor, D5 at every anchor; N6, D6
  mostly). Failures come with few rungs (D4, N7 at a and b).
- Joint solve, TSIZE 1e7: D6, D7 within 0.1–5% for seeds 1, 2 and 4; D8 within ~10% or worse. N chain: seed 1 N4, N5, N6
  within 4–7%; seeds 2–4 fail (slopes reach ~1e6 cycles/x, too many unwrap failures; ≤ 7 equations kept for 6
  unknowns).
- TSIZE 1e6 (seed 1): N4..N8 within ~10% (N7 −94882 vs −95140), D6 within 5%.
So the ladders turn into direct reads of the mid/top coefficients within ~10%, well inside the peel's range (fscan
±2e6 + exact rescore). More anchors would make the N chain robust for very far truths.
- Slope alias found and fixed. With the smallest rung at 2^-21, slopes beyond ±1e6 cycles/x alias by whole multiples of
  2^21 (control errors of exactly −2,097,0xx and −2·2^21). Real slopes reach that range (N6 at anchor b −1.47e6).
  Fixes:
  - 60 more ladder cases, kk = 22..26 for every anchor (`../gen_k3ladder.py`), plus a second anchor set g..l (0.7311,
    0.8741, 1.0309, 1.1789, 1.3021, 1.4129: 78 cases). Ladder total: 216 cases.
  - The joint solve is now a consensus fit (RANSAC over minimal subsets, inliers within 300 cycles/x modulo each
    equation's alias period, then LS on the consensus with the alias shifts applied).
  Controls at TSIZE 1e7, 12 anchors (`ladder_ctl_ransac.log`): with rungs up to 26, N4, N5, N6 within ~1% in 6/6 seeds
  (N7 in 5/6) and D5, D6, D7 within ~4% in 6/6. With rungs up to 21 only, 5/6 seeds solve.
- First real anchors (a, b; `ladder_real_ab.log`): slopes N3 +5.7e5 / −2.7e5, N5 −7.9e5 / +1.5e5, N6 −1.47e6, D5
  +8.7e5, D6 +2.4e5 / +4.1e5 cycles/x. These are TSIZE ≥ 1e7 offsets, i.e. far from ss_fc.
