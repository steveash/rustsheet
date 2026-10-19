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
    - Coarse stage values must come from model-free sideband triplets (W4: (E, D, B) gives mant(M(x))).

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
