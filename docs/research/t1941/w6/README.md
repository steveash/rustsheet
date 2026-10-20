# T-1941 W6: GAMMALN.PRECISE on (1.5,2]

All results come from committed oracle data (`kmyst-gam-*`), using exact x87-stored simulation (`lib6.py`: numpy
longdouble, then `astype(float64)` after every op). Scripts run from the repo root unless noted. Row dumps:
- `rows_all.py OUT lo hi`: every `GAMMALN.PRECISE(A<r>[+-k])` cell, including the P2 E/F columns.
- `rext.py ROWS OUT`: Excel's inner R per point.

## Settled
1. **The cut is at 1.5.** Near-1 kernel for x < 1.5; a new piece from 0x1.8p+0 (or 1.5+1ulp) up.
   - Structure detector (`corr.py`, `fcorr_split.py`): the |FFT| correlation of Excel's rounding residual
     (B - lnG)/ulp against a candidate's simulated residual.
     - Positive controls: ss_fc gives 0.78-0.85 on runs at 1.30-1.48; efC gives 0.80-0.96 on runs at 2.06-2.96.
     - The detector works even where the exact rate is only 20-30%. A wrong structure gives about 0.
   - On r203 (8,192 consecutive doubles centred on 1.5), ss_fc's correlation is 0.28-0.57 per 256-double block up
     to 0x1.7ffffffffffffp+0. From 0x1.8000000000000p+0 to 0x1.8000000000fffp+0 it is -0.09 to 0.16.
   - Whether x = 1.5 itself takes the new piece is open; q410 and a working kernel will settle it.
2. **(1.5,2] is neither kernel extended and not a composition.**
   - ss_fc and efC both correlate at about 0 on r711, p201-p204 and r712-r714.
   - Model-free (`modelfree_fc.py`): B's residual spectrum shares no lines with GAMMALN(x+1), GAMMA(x) or LN(x),
     nor with GAMMALN(x-1) from 1.81 up. All correlations are within 0.02.
     - Below 1.7, GAMMALN(x-1) correlates (0.30, 0.52), but only because GAMMALN(x-1) with x-1 < 0.7 is itself
       built as GAMMALN(x) - LN(x-1).
   - Unstored 64-bit variants of fl(K2(x+1)*(x-1) - LN(x)) through efC or grid_a also give about 0 (`comp.py`).
3. **The final ops are B = fl(t*fl(m*R)), with t = x-1 and m = x-2 exact, and R a stored double** (model-free
   reachability, `reach.py`, `reach2.py`).
   - fl(t*A) with A a stored double: 100% of Excel's B values are reachable. Base rates for random neighbours are
     0.756, 0.678 and 0.568 at 1.60, 1.65 and 1.70, and 0.317 at 1.825.
   - fl(m*S) sits at its base rate: 0.670/0.649 at 1.60 and 0.739/0.734 at 1.65.
   - A = fl(m*R): 0 of 16,384 points fail on r111, where random values fail 5.1%. The same holds at 1.85 (10%) and
     1.975 (1.6%).
   - fl(fl(t*m)*R) (0.943 against 0.916 at 1.825) and fl(m*fl(t*R)) (at its base rate) are excluded.
   - Controls: the x-1 form is 100% on [1.3,1.5) and the x-2 form is 100% on (2,2.3].
4. **There is an explicit (x-2) factor.**
   - B keeps relative accuracy down to x = 2 - 1ulp (|err| <= 1.24 ulp for k = 1..6). B(2-2ulp) = 2*B(2-1ulp)
     exactly.
   - Under the t-wrappers, the inner R at 2 - k*2^-52 starts at RN(1-gamma) for k = 1. It drifts +0.40 ulp per step,
     matching R'(2) = -0.1003, with +-2-3 ulps of stage noise (`near2.py`).
5. **R = lnG/((x-1)(x-2)) is a third rational, with Excel's own stage noise and staircase.**
   - R has 1-4 candidate doubles per point, 60% unique.
   - R - Rtrue per run (`rbias.py`): sd 1.0-1.5 ulps. The mean is -0.05 at 1.50-1.52, +1.15 at 1.62-1.66,
     +0.9..+1.0 at 1.72-1.81 and +0.75 at 1.88-2.0. That is a plateau staircase (playbook section 8.2), with a step
     in (1.52, 1.62).
   - The R-residual spectra have many lines of amplitude 0.3-0.5 ulp (`rspec.py`). Several drift smoothly with x,
     for example about 0.242 at 1.51 to 0.270 at 1.97, and about 0.326 to 0.372. That is consistent with a
     stored-stage Horner chain.
   - It is NOT Cody's form R = fl(c + fl(m*Q)): that predicts a clean fR = R'h/ulp(R) line, which is absent.

## With q401-q414 (344,448 points in (1.5,2))
6. The nested form holds everywhere: 0 misses on 344k points. Base rates are as low as 0.57 at 1.70 and 0.41 at 1.825
   (`reach2.py`).
7. **The R staircase** (`rstair2.py`, 512-double segments, SE 0.06 ulp; full table in the session scratchpad,
   regenerate it with the script):
   - -0.05..-0.10 on (1.503, 1.526);
   - a step to +0.34/+0.43 at 1.5285-1.5300, and to +0.85..+0.97 by 1.5327;
   - +0.75..+0.85 on (1.533, 1.612);
   - a step to +1.15 at 1.6120-1.6147;
   - then a smooth decline: +1.05 at 1.70, +0.90 at 1.80, +0.80 at 1.90, +0.72 at 2.0.
   - So there are two or three binade steps, plus a smooth function error of about +0.7..+1.1 ulp of R
     (+1.5..+2.5e-16 relative), as in K1.
8. **Sweep line tracks** (`segpeaks.py`, `hough.py`): about 16 crisp, nearly straight tracks of the unaliased
   frequency F(x), with |dF/dx| = 0.015, 0.0275, 0.06, 0.0975, 0.1225, 0.23, 0.48, 0.525, 0.63, 0.72, 0.915, 1.02,
   1.145, 1.155, 1.29 and 1.43.
   - Some tracks end or start abruptly (binade crossings), for example at 1.89.
   - Refined cubic tracks (`hough2.py`, 30 tracks): rms 3-6e-4 cycles per step; most have |F''| < 0.2. Several run
     unbroken to 1.999 (T0, T4-T6, T9, T16, T24). A product s*m in an m = x-2 chain has ulp -> 0 as x -> 2, so its
     line would chirp without bound. Tracks that stay straight up to 2 are therefore sum lines or raw-x products,
     which mildly favours raw x.
   - Straightness alone is not diagnostic: a fitted (8,8) raw-x chain also gives straight tracks within each binade.
   - None of the tracks belongs to the K2 (efC) or K3 (ss_fc) stage polynomials (`kscore.py`: hits equal the
     shuffled control).
9. **Generator habit** (`fitwin.py`): the exact rationals of ss_fc and efC against the true function.
   - ss_fc is accurate to about 2e-16 on [0.65, 1.55] and degrades fast outside it.
   - efC is accurate to about 1e-16 on [1.6, 3.3]; its error is one-signed, about -1e-16, and not equioscillating.
   - So K2's fit interval extends well below 2 (Cody-like (1.5,4)?), yet Excel does not use K2 on (1.5,2].

10. **Null-space search of a (6,6) chain, D0 = 1** (`nullde.py`): the 3e-16 function constraint allows only about
    1e-8 relative coefficient moves, so stage values are fixed. It explains 159 of 1,576 peaks. Excel's chain is
    therefore not a near-minimal (6,6) with that normalisation, and is likely over-parametrised like K3's (9,9).
11. **The pole/interval rule is unidentified** (`polescan.py`): no clean "relative LS on [a,b]" rule reproduces Excel's
    K3 or K2 poles. The best K3 match is (9,9) on [0.75,3] at 0.15 rms log-distance.

12. **Raw x vs m = x-2: raw x favoured, with controls** (`synth.py`, `synth_m.py`).
    - In an m-variable chain, the products s*m vanish as x -> 2, so their lines die. The x-2 synthetic, a (9,9) LS
      of R on [1.2,2.6] in m, shows strong peaks (amp > 0.35, per 0.1 band from 1.5) of 614, 287, 127, 94 and 34.
      Its R noise sd falls from 2.5 to 0.7 ulp.
    - Excel: 335, 309, 394, 288 and 250, with sd from 1.4 to 1.1.
    - The raw-x synthetic (same interval): 221, 275, 306, 314 and 451, with sd 1.2-1.5.
13. **Why the coefficients are hard: quantified on a synthetic (9,9) control** (`synth_truth99.txt`; data simulated
    on the real sweep grid and re-extracted like Excel's).
    - Function null space (`nullspace.py`, exact in mpmath): the singular values decay about 40x per step. With a
      1e-16 function tolerance, about 7 directions are open. Measured in units that change stage values by 1e-3, they
      span 6 to 2e12 units. The function pins no mid or upper stage, and even D's shape on [1.5,2] varies by 5%
      under D0 = 1.
    - Analytic line laws (product P'h/ulp, sum, triangle) explain only 2 of the 30 synthetic tracks. The rest are
      intermodulation of stored-multiplicand products. A DE on the line-law objective finds a wrong chain that scores
      above the truth (0.406 against 0.395) (`polede.py`).
    - The exact-simulation spectral objective (`simobj.py`) scores 0.92 at the truth. Under log-pole perturbations
      it gives about 0.3 at 1e-3, 0.03-0.07 at 1e-2 and 0 at 1e-1. In pole space with N from LS it is degenerate:
      DE reaches 0.44 at wrong poles, while the truth scores only 0.33 through the same LS (`simde.py`).
    - Conclusion: no global search from function plus lines can find Excel's chain. Stage values must be read at
      anchors (ladders q409 at 2-, q410 at 1.5+, r203, r204, c304) by exact search in stage-ulp coordinates, as W1
      and W2 did, and then joined. That is K1-K3-scale work.

14. **R is not a plain polynomial** (`polytest.py`, `polyctrl.py`).
    - Forms tested: the Chebyshev interpolant on [1.5,2], degrees 12-18, as Clenshaw in z = 4x-7, Horner in z, in
      2x-3.5, in x-c for c = 1..3 and in raw x.
    - Exact-sim coherence on the real data is -0.02..+0.02 for all of them.
    - Control: Chebyshev-extrema coefficients (4.6e-3 different) scored with the root-node ones give 0.96-1.00.
    - A polynomial has no null space, so this is a clean rejection.
    - Clenshaw N/D fits (5,5)-(9,9) also score about 0, but only (5,5) is meaningfully rejected (null space).
15. **How far Excel's kernels sit from plain LS fits** (`nullspace.py`, used as a calibration):
    - K2 efC lies in the linear null space of a (9,8) LS fit on [2,3], at <= 930 units of 1e-3 stage change in the
      softest direction.
    - K3 ss_fc is nonlinearly far: its D shape differs, with N stages 3-100x apart under D0 = 1.
    - So a blind null-space search for R is not feasible. Coarse stage values must come from the line structure
      (W4), and then a phase solve (W2 pipeline, `w2/wa/`) takes them to ulps.

16. **Stage-line laws** (with W4's product-sideband law; `stagelaws.py`, `peaklaw.py`, `lawobj.py`).
    - On the synthetic (9,9), the full law set explains 68.5% of 2,149 per-segment peaks (amp > 0.3, tol 4e-4),
      against a shuffled-x null of 23.9%. The set: S, T, P, product sidebands P_j - n F_{S_{j+1}}, sum sidebands
      S_j - n F_{P_j}, and quotient lines F_R - i F_N - k F_D.
    - Dominant families: n = 1 product sidebands of P0-P4 and the triangles T3-T5. Plain P'/2^e lines are almost
      absent.
    - Cubic Hough tracks stitch lines together, so per-peak matching is the right unit.
    - The law objective is a needle. From the LS fit to the truth along the exact null space it scores 0.27 at 0%
      and at 90%, 0.63 at 99% and 0.69 at 100%. It needs stage values to about 1e-4 relative before it can verify
      anything.
    - Single-track assignment enumeration (`enumtrack.py`) is non-identifying: 8 assignments fit synthetic T1 at the
      noise floor. The true one only has the smallest shift from the LS fit (0.10 against 0.46-2.4).
    - The assembler takes stage-value rows. Control: S1..S5 of both chains at 1e-6 noise recover S0-S5 to 1e-6.
    - The assembler also takes combination rows (kind 'comb': sum_i n_i s_{j_i}'/2^E_i). Control: n = 1 sideband rows of
      P0-P4 of both chains (24 x, 3e-4 noise, exponents per point) recover S0..S4 to 1e-5..1e-4.
    - The LS fit's predictions for the dominant laws are 0.02-2.8 cycles off the synthetic truth: only N.T4, D.T5,
      N.P3n1 and D.P4n1 are within 0.06. So an EM bootstrap of the assignment from c0 is not viable.
    - Coarse stage values must come from model-free sideband triplets (W4: (E, D, B) gives mant(M(x))).

17. **Long-run pipeline: from no knowledge to about 1e-7 stage accuracy on the synthetic control** (`longspec.py`,
    `grow.py`, `enumlong.py`, `jointtest.py`, `emtracks.py`, `lawrows.py`; synthetic (9,9) simulated at the 24
    L101-L124 starts, 65,536 doubles each).
    - Precise peaks: 59% of the top-25 per run match a law within 3e-5, against 2.2% chance.
    - Model-free track growing across the runs: 37 tracks. 9 are single-law tracks at rms 2-7e-7.
    - One track alone is NOT identifying: the truth ranks first, but wrong laws fit within 1.3x.
    - Joint beam search over 4 tracks: the true combination fits at 4.2e-7, the next at 4.4e-4 (1000x gap). Two tracks
      already separate it.
    - EM bootstrap from the 4-track chain: it accepts exactly the 9 genuine tracks, all correctly assigned. Mid and
      deep stages reach 5e-8..1e-6 (top stages 1e-5..5e-3); the law objective goes from 0.27 to 0.62 (truth 0.685).
    - Risk for Excel: candidate exponents come from the LS fit c0. If Excel's stages differ from c0 by >2x (as K3's
      do), they can be wrong. In that case enumerate the exponent offsets too.
    - Next: the same pipeline on Excel's L101-L124, then ladder-phase unwrapping (q409/q410/q415-q420) from 1e-7 down
      to ulps.

18. **Per-run line phases** (`phasediff.py`, synthetic control). Data-vs-sim complex DFT phase at each identified
    line, per long run, matches the exact prediction to about 0.005 cycles (amplitude ratio 1.0 +- 0.07):
    - frac(dS/u) for an S line;
    - frac(dS/(2u)) for T;
    - dP/u_P - n dM/u_M for product sideband n.
    - The predictions use mpmath polynomial stage values at the run start.
    - Reference the predicted phase at the run centre: a frequency mismatch dF otherwise adds dF(N-1)/2.
    - Then T lines agree to 0.001-0.02 cycles. The n=1 sideband D.P2n1 carries a systematic -0.055 cycle offset.
      Plain S lines are unreliable (0.04-0.23 off, unstable amplitude), which matches W2's large-offset finding.
      Use T lines for phase rows, sidebands with a fitted constant, and not S lines.
    - Each (run, line) therefore gives a congruence sum_k x^(k-j) (ulp(c_k)/u_S) m_k == phi (mod 1) in the integer
      coefficient offsets m_k. With 24 runs x ~9 lines that is a lattice problem: about 1,600 bits of information
      against about 820 bits of unknowns.

19. **First Excel pass, on W4's 14 precise runs: not decisive** (`enumlong.py` with exponent offsets, `jointtest.py`,
    `xcheck.py`).
    - A and C each fit many laws at the 1.7e-6 floor. D fits no single law (best 3.3e-4).
    - Joint A+C: the best is C = D.T5, A = N.T5 (3.4e-6), the next 8.4e-6 (a 2.5x gap; the synthetic gap with 4 tracks
      was 1000x).
    - Predictions of W4's 95 unassigned peaks from the top hypotheses are at chance (4-10 explained against a null of
      5-10).
    - At least 4 independent precise tracks are needed, so this waits for L101-L124.
    - Automatic track filter (synthetic): the 4 long non-law tracks have best single-law rms 0.1-0.3, against
      2-7e-7 for law tracks. So only tracks whose best fit is near the precision floor are kept. By that rule Excel's D
      (3.3e-4) is a non-law line (mix or sideband with r flipping), and A and C (1.7e-6) are law lines.

20. **Positive control: BIT-EXACT recovery from zero knowledge** (synthetic (9,9) truth, Excel-format data on the real
    grids):
    1. The long-run pipeline (item 17) gives an EM chain with stage values at about 1e-7, i.e. 1e9-5e14 coefficient
       ulps off, at 19% exact R.
    2. Lattice pass 1 (`latsolve.py`: CVP by LLL+BKZ-20 in fpylll). Inputs: per-run phases (0.01-cycle noise) of T4,
       T5 and the n=1 sidebands of P0-P4 of both chains on 24 runs, plus exact function rows. Result: N1-N4 and
       D2-D4 exact, N5/N6/D5/D6 within 69-653 ulps, the top within 4.5e3-1.6e7. 41% exact R.
    3. Lattice pass 2, adding T3, T6, T7 and sidebands P5-P7: everything exact except N9 (6 ulps), N1 (2) and the
       line-invisible constants N0 and D1 (function-only, 2.2e4 and 2e3 ulps).
    4. `pairscan.py` along the function-weak (N0, D1) direction: 99.95%.
    5. `climb6.py`: 100.000% exact R and exact B on all 134,336 points. The remaining coefficient offsets (N0 -59,
       N1 +2, N9 -6, D1 -231 ulps) are output-invisible.
    - The read-back bug found on the way (int(SC/bound) truncation) is fixed.
    - For Excel the open inputs are: >= 4 jointly assigned precise tracks (L101-L124), per-run Excel phases at those
      lines (`phasediff.py` with Excel's R in place of the synthetic), and the same lattice, scan and climb.

21. **Hardening for Excel-like conditions** (coordinator's list):
    - Phase-precision limit of the lattice (`latsolve.py`, from the EM chain):
      - with the 12-line set and BKZ-20: works at 0.01 cycles, fails at 0.02-0.08;
      - with the full 25-line set (T3-T7 and n=1 sidebands of P0-P7, both chains) and BKZ-30: at 0.04 cycles every
        coefficient is exact or within 1-24 ulps, with only N0/D1 (function-only) off by 2.4e4/2.2e3 (pair-scan
        range);
      - the 0.02 run crashed in fpylll (infinite loop in Babai, double-precision GSO); use float_type 'mpfr' for
        Excel.
    - A far, K3-like truth (`synth_truth99_1_6.txt`: (9,9) LS on [1,6]; stages 0.06-0.27x the [1.5,2] fit, so
      exponents 2-4 off):
      - its non-null residual against the narrow (thr 1e-15) null space is 6e-5, so the fast enumerator
        (`fastenum.py`) needs the wide null space (thr 1e-8, 10 dims) with soft function rows (ftol 3e-16);
      - then single-track floors separate law tracks again (2e-7 against 1e-4..0.2);
      - but only 2 of the top 16 tracks are single-law (N.T4, N.P3n1, eoff -2), and an N<->D-swapped pair fits almost
        equally (3.4e-7 against 3.5e-7);
      - robust per-peak EM (`peakem.py`: global eoff, trimmed refits) from the correct seed reaches stage errors of
        2e-4 (41 consistent matches, max res 8.5e-7) and stalls. The wrong seed stays at 650% error, with less
        consistent matches (max res 4.8e-6).
      - So a 2-track start is marginal; Excel needs >= 4 law tracks or sideband-difference (comb) rows.
      - The N/D ambiguity is resolved by the R staircase (`crossings.py`). On the far truth's sweep, the 5 staircase
        steps all sit at binade crossings of the correct-seed EM chain (5/5, window 0.003). The swapped-seed chain
        explains 2/5 (chance); so does an unrelated chain. Apply the same check to Excel's steps (1.526-1.533,
        1.612-1.615) and to D's end at 1.889.
      - Negatives:
        - adaptive (median-based) trimming in peak EM diverges (stage error 0.2-0.5);
        - law-track EM (`lawem.py`: per-law nearest-peak sequences, accepted if smooth in x) accepts nearly every law
          at 24/24 inliers, because a smooth wrong prediction near a real track looks smooth too;
        - so EM must gate on absolute residuals (per-peak EM with a 5x-tol trim, which reached 2e-4).

22. **The far, K3-like control is also BIT-EXACT** (truth `synth_truth99_1_6.txt`, stages 0.06-0.27x the [1.5,2] fit,
    exponents 2-4 off):
    1. Wide fast enumeration with exponent offsets -4..+1, plus the skip beam: 2 law tracks found (N.T4, N.P3n1,
       eoff -2), with an N<->D-swapped tie.
    2. The staircase/crossing discriminator picks the right seed (5/5 steps against 2/5).
    3. Per-peak EM: stage errors 2e-4.
    4. Track EM with the exact mpmath assembler: 7 tracks (adding D.T5 and D.P0..P4 n1). S0-S4 reach 2e-7..1e-6.
       The fast linear subspace stalls at 2e-4 because of its linearisation, so the exact assembler is required here.
    5. Lattice passes 1 and 2 (full line set, BKZ-30, 0.01-cycle noise): all exact except N0, D1 and N9 (-12 ulps).
    6. `pairscan.py` on (N0, D1): 99.99%.
    7. `climb6.py`: 100.000% exact R and B on 134,336 points (`applym.py` applies lattice offsets).
    - Both controls (near and far truths) now succeed from zero knowledge.
    - `run_xl.py` (the driver for steps 1-4) reproduces the far-truth result blind. Staircase ranking chooses the N
      seed (5/5 against 2/5 and 0/5 for the D seeds), per-peak EM keeps 71 consistent matches, and exact track EM
      accepts the same 7 tracks at 1-3e-7.

23. **Excel L101-L124, first pass** (24 runs x 65,536 doubles, merged cleanly; 39 tracks).
    - Floors of the 18 longest tracks under (9,9): 9 law-like tracks at 1.8-6.8e-7 (0, 2, 4, 8, 9, 13, 14, 16, 17),
      8 mixed (0.004-0.19), one borderline (1.1e-5).
    - None of the tracks is an outer-op line (A = fl(m R), B = fl(t A), or their sidebands; best rms 0.04-0.55), as
      expected for spectra of the extracted inner R.
    - Fast joint skip beam, laws that fit jointly by degree:

      | Degree | Law-like tracks | Fit jointly |
      |---|---|---|
      | (7,7) | 0 | 0 |
      | (7,8) | 4 | 1 |
      | (8,7) | 5 | 1 |
      | (8,8) | 6 | 1 |
      | (8,9) | 9 | 1 |
      | (9,8) | 9 | 2 |
      | (9,9) | 9 | 2 |
      | (9,10) | 10 | 2 |
      | (10,10) | 10 | 2 |
      | (10,9) | 11 | 3 (D.T5, N.T4, N.T5; rms 1.5e-6) |

    - Low degrees are clearly worse. From (9,8) up, the fast solver caps at 2-3 jointly fitted tracks, as it did on the
      far synthetic truth, so the exact assembler (`mpjoint.py`) is the real test.
    - Excel's staircase detector gives 7 steps, of which 3 are real (1.527, 1.531, 1.613); the rest are
      single-segment noise. The driver's crossing ranking was inconclusive (best 4/7).

24. **Excel, measured-phase lattice, pass 1** (`latsolve.py ... measured`, `xlphase.py`, `applym.py`).
    - Input: the 6-track EM chain (`xl_EM5`). 300 measured per-run phase rows (amplitude ratio 0.7-1.4; coherent lines
      D.T5, D.T6, D.Pn14-16, N.T4-6, N.Pn14-15). BKZ-30, MPFR-200 LLL (fpylll needs `method='proved'` for MPFR).
    - Recovered offsets are 1e12-1e15 ulps, the same range as the far control's true offsets from its EM chain
      (1e9-1.3e15).
    - Applied to Excel R on 20,000 points: exact 20.2% -> 25.6%, median relative R error 2.43e-16 -> 2.31e-16.
    - **Retracted: spurious.** The offsets satisfy the phase congruences mod 1 but move every line by 24-3,900 DFT
      bins (`latcheck.py`; the frequency gradient in `latsolve.freq_rows` matches a direct `stage_F` difference to
      every printed digit). Re-measured phases against that chain confirm it: the coherent lines drop to amplitude
      ratio <0.3. The exact-R gain came from the function rows alone.
    - Fix: frequency-hold rows (`FREQ=w`: phase-equivalent weight per cycle/sample; 2621 makes half a bin cost the
      same as 0.02 cycles of phase). Measured-phase lattice runs must pass `latcheck.py` (all lines < 0.5 bins).
    - With frequency hold (FREQ=2621), pass 1 on Excel: lines move 0.02-0.4 bins (Pn16/17 about 1); exact R 20.2% ->
      27.5%, median relative R error 2.43e-16 -> 1.30e-16. Re-measured coherence against the new chain: D.T5, D.T6,
      D.Pn14, D.Pn15, N.T5, N.Pn15 stay 24/24 runs; N.Pn14 22 -> 24; N.T4 21 -> 20, N.T6 21 -> 17; D.Pn16 20 -> 8. So
      the gain comes with coherent lines.
    - Pass 2 (re-measured phases, `latpass.sh`): 20.3% exact, 2.43e-16, worse than pass 1. N.Pn11/12/16/17 drift
      1.5-2.7 bins.
    - Bug: `applym.py` stepped the bit pattern, which is wrong once a coefficient crosses a binade (pass 2 crossed one).
      It now adds m*ulp(c) as a value.
    - Control without frequency hold (far truth, from its EM chain, 402 rows): offsets wrong by ~1e14 and lines moved
      by hundreds to thousands of bins, the same failure as Excel's retracted pass 1.
    - Exact climb (`climb6.py`, 20,000 points, steps up to 2^50 ulps) from the 27.5% chain: 32.1% after round 0, then
      flat.
    - **Control with frequency hold fails too** (402 rows): offsets no closer to the truth, exact 28.5% -> 26.3%. So
      Excel's 27.5% / 32.1% is a better fit, not evidence of the true chain.
    - Cause: the measured phases. Against the ideal (truth - candidate) phases on the control (`phasediag.py`):
      accurate lines (median error 0.002-0.008 cycles) are N.T4, N.T5, D.T5, D.Pn13-14, N.Pn12-14; Pn10-12 lines are
      biased by about 0.04; T2/T3/T7, Pn16/17 and others are junk (about 0.25) yet pass the 0.7-1.4 amplitude filter.
    - Selector usable on Excel: a line's median |log amplitude ratio| < 0.1 keeps every accurate line except N.Pn12
      (0.16) and rejects every junk line (> 0.45). Run-to-run phase smoothness does not discriminate.
    - Excel clean lines by that rule: D.T5, D.T6, D.Pn14, D.Pn15, N.T4, N.T5, N.Pn14.
    - Clean lines alone are too little information: ideal phases + noise on only the 8 clean control lines (24 runs)
      recover the truth at sigma 0.005 (max error 5e7 ulps) but fail at 0.01 (1.5e12).

25. **Run ladders as lattice rows** (`ladder.py`, `ladpass.sh`, `latsolve.py` LADDER/LADW/LADRES/LADDF).
    - Reader: per anchor and law line, Hann demodulation at the candidate's frequency, phases referenced to each run's
      centre and to a reference run (K = 0, or the j = 46 '+' rung for the dyadic set), then W4's unwrap. Output: the
      line frequency F at the reference-run centre, mod 1. Each kept (line, anchor) is a mod-1 congruence row on the
      verified frequency gradient, weighted 3e8 x a phase row. BKZ needs MPFR too (double Babai loops).
    - Layouts (confirmed): dyadic q421-q433 (a = 1.625, 1.75, 1.875; x0 = a +- 2^-j, j = 22..46 step 2; 4,096-runs;
      no K = 0 run) and non-dyadic q440-q453 (a = 1.6341, 1.7718, 1.9253; c = 1.37; K = 0 runs in q440a).
    - Synthetic far truth on these layouts, read blind against the EM chain: unwrap max|res| < 0.15 keeps 35 + 33
      rows, all within 4.6e-10 of the truth, none bad; W4's 0.06 also keeps no bad rows but drops half the good.
    - Control pass 1 (68 ladder + 273 clean phase rows): N0-N4 = 657, 1, 34, -10, 0 ulps; D1-D4 = 155, -5, 1, 0;
      middle 73-6.5e4; top 5e5-8e7. Pass 2: D1-D9 within 1.9e5 (D3-D5 exact), N0-N7 within 9e4, but N8 -4.4e6 and
      N9 6.6e8; climb then 80.2%. The top pair needs more than these rows (non-dyadic ladders).
    - Excel dyadic ladders (2,254,016 rows, no run missing): 22 of 84 (line, anchor) pairs pass the 0.15 gate, all
      from the coherent L-run lines (D.T5, D.T6, D.Pn14, D.Pn15, N.T4, N.T5, N.Pn15) plus N.Pn13. Against the EM chain
      six of them agree within 0.01 bins (of a 65,536-run) at every anchor: independent confirmation of those stages.
    - N.Pn13 sat at a constant -9.01 bins: a mislock that broke the first Excel pass (5.7% exact, the lattice gave up
      the function). Agreement gate LADDF: drop rows more than 1.5 bins from the candidate (good control rows are
      within 0.93, median 0.013). Gated passes 1-2: 27.1%, 26.7% exact, median 1.30e-16.
    - Structure of the gap: a stage-j line's frequency depends only on c_{j+1}.. (never c_j), and Excel's coherent
      lines are all stages 4-6, so the low coefficients were constrained by the function rows alone.

26. **Excel's low-stage lines exist; the D chain is consistent from stage 2 up** (`lowfit.py`, `xl_lowjoint.py`).
    - The L-runs have 39 precise tracks (smooth to 2-5e-7); the chain explains 6. Linear fit of each unexplained
      track to every law, with the upper stages held, then the exact mp assembler: track 27 = D.Pn12 and track 31 =
      D.Pn11 each fit to noise (9e-7) alone but not together (8.6e-6); tracks 28, 37, 3 fail exactly.
    - The dyadic ladders decide it. Read against the track-27 chain: D.Pn12 (-0.00, +0.01 bins), D.Pn13 (+0.12 at all
      three anchors, not used in the fit), D.Pn14/15, D.T5/T6 within 0.24. The track-31 chain misses D.Pn12/13 by
      ~15 bins. Against the track-27 chain the 24-run phases also show D.Pn12/13 coherent for the first time.
    - N.Pn13 unwraps cleanly in every chain with an offset that follows the N low coefficients (-9.0, -5.1, -1.66
      bins), but admitting it (LADDF 2) breaks the function (3.4% exact); with LADDF 1 the pass holds (26.1%, 1.31e-16).
    - Low offsets still swing ~1e15 ulps between passes. Given the coefficients j >= 3, the function rows pin
      N0-N2/D1-D2 almost exactly, so the swing is compensation for the uncertain top pair. Order: non-dyadic ladders
      for N8/N9/D8/D9, then a W5-style peel (`w5/p5/fscan.py`) on the low coefficients over a narrow range.

27. **Non-dyadic ladders, pass 4, and the finishing step** (`kern.py`, `nullscan.py`, `fscan6.py`, `top6.py`).
    - Out-of-sample check: the pass-3b chain (fitted with the dyadic ladders only) predicts the non-dyadic ladders
      (q440-q453, anchors 1.6341, 1.7718, 1.9253) to 4e-11..4e-9 cycles on D.Pn12-15, D.T5/T6, N.T4/T5, N.Pn14/15,
      including at 1.9253 above D's 1.89 crossing. N.Pn13 sits at a constant -4.2e-6 cycles; N.Pn12 is 2 bins off.
    - Pass 4 (both ladder sets): every variant breaks the function and lands at 12-14% exact (consistent phases;
      lgamma or Excel-R-anchored function rows; ladder weight 1e6..3e8 gives an erratic 13-29%). Function + ladder
      rows pin only ~5 of 19 directions (1-sigma moves of 1e7..4e14 ulps along the rest). A first pass-4 attempt that
      reused phases measured against a different chain is void (phase rows are relative to their chain).
    - Control: pass 2 + climb 80.2% (low within 153 ulps; N9 -6.6e8, N8 4.4e6, N7 -8.9e4); pass 3 regresses the low
      coefficients (N0 -2e9). Along the function-null direction even the true N9 scores <= 79.1% exact: exact R has
      no peak at one coefficient while others are off.
    - W5's FFT peel ported (`fscan6.py`, with downstream sum/product sites): needs the stages above J exact (peaks at
      k = 0 otherwise); N9's signal (via S8 rounding, ~0.02 ulp of R) is noise-level.
    - W5's top objective (`hypo.c` via `top6.py`): truth 9.8 vs base 4.3 on the 24 control runs; capture N9 ~ +-300
      ulps, N8 ~ +-1 ulp. A grid over the control's top error (N9 6.6e8, N8 4.4e6) is infeasible; the top needs a
      prior within ~1e4 (N9) / ~10 (N8).
    - Top-weighted probes near 2- would not supply it: on the synthetic truth, ladders at 1.962/1.9745/1.9871 read
      37/84 rows but no top-stage line (T6+, Pn15+), like the other sets.
    - K2 support: `KERNEL=k2` selects R = lgamma/(x-2), B = fl((x-2) R) (`kern.py`; k6 verified identical). K2
      synthetic ((9,8) raw-x LS truth, `fitrat.py`): ladders read only N.Pn14/D.Pn13 (4 of 108 rows) even against
      the truth.

28. **Top-down route to 99.4% on the far control** (`scan6.py`, `apply6.py`, `fscan6.py` SITES=P, `peel6.sh`,
    `bump.py`, `stagerr.py`, `pairscan.py`, `climb6.py`; data: 12.3M synthetic points on the coordinator's w6t layout
    plus the 24 L-runs).
    - Sites: S8's rounding reaches R at ~0.001 ulp (invisible); S7 quadratic gives Z 5.0 at the truth on 12.3M points;
      S6 cubic (u(S6) = 16 u(S7)) gives Z 25. Per-run phases are useless at every site (rms 0.24 cycles).
    - 1. S6 4-D scan (a mod 1 x b by FFT x c, d grid; scan6 J=6 DEG=3): truth found at Z 20.5 (Z 21.8 with c step 1,
      d step 4; 32 ms per (c, d) pair, ~1.3 h full range on 4 cores). On the 2.6M points Excel had before the fill
      the truth wins only locally (Z 7.1).
    - 2. `apply6.py` maps the S6 polynomial to N6..N9: the S6 error becomes a constant (1533.4 u), i.e. one integer.
    - 3. Whole-u shifts by products-only m-scans (`fscan6.py` SITES=P NS=1, on P_{J-1}): N6 +1534 (Z 5.8), N5 -341 =
      truth (Z 15.9), N4 0 = truth, N3/N2 stage-exact. Including the S site or a second product locks near k = 0.
      N1's ulp is 1/256 of u(S1), so its m-scan is flat (pick -71 was wrong): sub-stage-ulp coefficients are left to
      the end.
    - 4. S7 quadratic scan in a small window (error now ~200 u7): Z only ~3.7, but the top hits cluster on the truth
      (b -118.5 vs -118.6, c 71 vs 71.5); apply6 + m-scans N7 (+205), N6, N5. N stages S7..S0 within 0.3 u.
    - 5. Transplant diagnostic: true N0/N1/N2/D1 -> 99.3%; all true top (N7-N9, D6-D9) -> 81.3%. N8/N9 barely matter
      once S7 is right (S8 enters R only through ~0.002-ulp roundings); W5's hypo.c (N8, N9) grid is too weak anyway
      (truth Z ~2.7 on 500 runs).
    - 6. The low deficit is the coupled pair (N1, D1) (alone 66.9% / 75.2%, together 95.8%): `pairscan.py` N1 D1 ->
      96.0%; `climb6.py` -> 99.49% (sample), 99.41% on all 1.57M L-run points.
    - Left: D.S7 (100-134 u) / D.S8, N.S8 (~150 u). D.S7 scan running.
    - Unstored-x87 question (W2's K2 finding): `oprep6.py` flags Excel's N.S1/P1/S0/P0, but a stored truth with N0 or
      D1 off gives the same kind of fingerprint, and the missing Excel lines include D.Pn10/11, which unstored N ops
      do not remove (synthetic with N.P0/S0/P1/S1 unstored loses exactly N.Pn10/11). Open; the arbiter is the
      top-down peel on Excel. KEEP support: `synth.horner_keep/simR`, scoreR, climb6, oprep6.

29. **Excel S6 scan: no detection; Excel's start is the blocker** (`scan6.py`, CLO/CHI slices).
    - Control finish (item 28 continued): D.S7 quadratic scan (Z 14.1, truth found), m-scans D7..D5, climb -> 99.95%;
      S8 exact-R grids (`grid8.py`) + a 100k-point climb -> **99.97% exact on all 1.57M control points** (both S8s stay
      ~130-170 / 16-20 u off the truth, on output-equivalent plateaus).
    - Excel: S6 4-D scan from xlL3b on Rall + 149 w6t cases (5.03M rows, 1,432 runs), window b +-2e3, c +-1e3, d +-600:
      best joint Z 4.5-4.9, scattered (null sd 207 vs 82 on the control).
    - Cause: Excel's S6 is pinned only to ~1e5-1e6 u(S6). Excel chains disagree on N.S6 by 1e6..1e11 u (the closest
      family by 1e6..5e7 u); the ladder + function rows leave >= 8 weak directions in N5..N9, each a 1-sigma move of
      0.9e5..4.6e5 u(S6), so there is no low-dimensional null space to scan instead. The control's lattice pass 1 had
      N.S5 exact and N.S6 within ~300 u because its phase rows pinned stage values; on Excel every phase-using pass
      disagreed (S5 differs by 1e6..1e9 u between passes).
    - So the Excel route needs value constraints that work: either the stage phases are mis-modelled on Excel (phase
      convention or an op the model gets wrong) or the phase rows need a better start. Next: measure Excel's N.T5 /
      D.T5 anchor phases on the K = 0 ladder runs (q440a) and test whether any smooth S5 shift explains them.

30. **Excel's line phases are consistent with stored raw-x Horner** (`phasecons.py`: small CVP over only the
    coefficients the chosen lines depend on; rows = 24 L-run phases + ladder F rows; no function rows).
    - N4..N9 from N.T4, N.T5, N.Pn14, N.Pn15 (91 phase + 17 ladder rows): BOUND 1e9..1e13 leaves 0.07-0.20 cycles;
      at BOUND 1e15 / 1e16 (same solution) 0.004-0.015 cycles and every F row within 0.03-0.18 sigma, the control
      level. xlL3b was ~1.8e14 ulps off in N9.
    - D2..D9 from D.T5, D.T6, D.Pn12-15 (144 + 32 rows): 0.003-0.009 cycles (D.Pn12 0.058), F within 0.7 sigma.
    - Controls: near start (pass-1 chain) 0.003-0.010 cycles; far start (EM7) at BOUND 2e15 also consistent.
    - Uniqueness: from the EM chain instead of xlL3b the absolute N5..N9 agree to 0 / 30 / 844 / 6.3e4 / 1.6e6 ulps.
    - Checks requested by the coordinator: free per-line constant offsets do not explain the small-bound misfit (no
      0.5 / 0.25 structure); each x-half alone fits only somewhat better.

31. **The missing parameter is D0: Excel's (1.5,2] D0 is not 1** (`augls.py`, `lowsolve.py`, `synth.nrm`).
    - With the phase-pinned N5..N9 and D3..D9 and D0 = 1, no low coefficients reproduce Excel's R: residual 5.2e-15
      (lgamma rows 5.9e-15), smooth in x and needing degree 8-10; upper moves of ~1e9 ulps would be required.
    - Augmented LS (one extra parameter): k (R = k N/D) 3.24e-16 with k - 1 = -3.6e-7; c 3.41e-16; c(x-2) 3.83e-16;
      c(x-1) 3.76e-16; e 5.81e-16; extra N10 4.74e-16, N11 5.16e-16, D10 4.52e-16.
    - With D0 free and no normalisation: 1.6e-16 rms vs Excel's R (the data noise; xlL3b 1.8e-16) and exact R 35.5%,
      the best so far. The LS D0 itself (~0.0016) is not meaningful (the low block has a function-null direction).
    - Every tool normalised by D0, which is exact only for +-2^k: `synth.nrm(D)` returns D0 only when it is +-2^k, else
      1; patched into 26 scripts; D0 = 1 chains are unchanged (control 99.97% unchanged).
    - D9: the phase fit gives D9 = -1.4e-13 (xlL3b 1.6e-5), i.e. D is effectively degree 8 at the line level; D9 enters
      only D.S8 = D8 + D9 x at ~1e-13 relative and is left free for the exact scans. The (9,9) machinery covers (9,8)
      with D9 = 0 as a special case, so the controls' assumptions are unaffected.
    - Phase / ladder files and the two chains: `data/`.

## Negative so far
- Relative-LS rationals of R on [1.5,2] in raw x, with D0 = 1:
  - maximum relative error 7e-16 for (4,4), 2e-19 for (5,5) and 7e-23 for (6,6);
  - B exact 14%, 28% and 30%, with structure correlation about 0;
  - predicted stage lines match 6-13 of 108 observed peaks, against about 19 expected by chance (`rtest.py`,
    `lines.py`).
  - So fitted stage values are nowhere near Excel's: there is the null-space and normalisation problem, as for
    K1-K3.

## Next (needs probes q401-q414, `gen_q4.py`)
- Sweep q401-q408 (256 x 512 doubles):
  - map the R staircase; each step is a binade crossing of an Excel stage, which gives model-free stage values;
  - track the R lines across (1.5,2), so their frequency curves give the stage functions (W4 law method);
  - get Excel's measured function, for a degree decision on the staircase-corrected curve.
- Ladders q409 (2-) and q410 (1.5+): stage phases at the two ends, and the x = 1.5 assignment.
