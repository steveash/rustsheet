# T-1941 research notes: Excel GAMMALN, GAMMA and LN

Data: `eval/suites/probes-numeric/cases/kmyst-gam-*`.
- r1xx: 16,384 consecutive doubles from irregular starts.
- r2xx: 8,192 consecutive doubles centred on candidate boundaries.
- Columns: A = x, B = GAMMALN.PRECISE(x), C = LN(x), D = GAMMA(x).

`extract_runs.py OUT` dumps the rows as "x_hex B_hex C_hex D_hex case", with nan for errors. Run it from the repo root.

## Findings so far
- **LN is bit-exact (x87 FLDLN2; FYL2X).** LN(x) = RN53(RN64(RN64(ln 2) * log2(x))).
  - It matches 114,687 of 114,688 points (`lnmodels.py`).
  - The one miss, x = 0.612354100000274, has its exact value 0.008 of a 64-bit ulp from a 64-bit midpoint, an
    FYL2X near-tie like EXP's F2XM1 near-ties (T-1778).
  - Plain double rounding RN53(RN64(ln x)) gives 114,674; correctly rounded gives 114,658.
- **GAMMALN for tiny x is fl(-LN(x) + fl(x*(-gamma)))**, with LN the model above and gamma Euler's constant
  as a double. That is 16,384 of 16,384 points near x = 1.37e-9 (`small.py`). It is Cody's ALGAMA form,
  -log(x) + x*(D1 + x*R(x)) with D1 = -gamma, where x*R is negligible at that x.
- **GAMMA is a separate kernel**, not EXP(GAMMALN).
  - Near x = 1.37e-9 it runs 1.5-2.4 ulps high. The implied numerator g*x is about 1.2*2^-52 above
    Gamma(1+x), consistent with a stored Gamma(1+x)-like value about 3 ulps off, then divided by x.
- **GAMMALN for x >= 8 is bit-exact: Stirling with exactly 10 Bernoulli terms (W3, verified by the
  coordinator).** Every op is fl53(fl64(.)):
  - corr = Excel LN(y); a = y - 0.5; b = a*corr; d = b - y; e = d + 0.9189385332046727 (ln sqrt(2pi));
  - z = 1/y; z2 = z*z; r = B10; then r = r*z2 + B_k for k = 9..1; s = r*z; GAMMALN = e + s;
  - B_k = B_2k/(2k(2k-1)) correctly rounded to double (1/12, -1/360, ..., -174611/125400).
  - Score: 291,957 of 291,957 points (`gammaln_big_check.c`, `USEC=1 NT=10 ./glc rows.txt 8 1e308`), from 8 to 7.8e299. That includes
    r118-r125, r206-r212 at x >= 8, r301-r303 (up to 7.8e299), r402-r403 and the r401 sweep.
  - 11 terms give 291,931 (12 misses at 8.03 alone), and 9 terms fewer. Cody's ALGAMA coefficients and his divide-by-y^2 form are not
    used. The cut is exactly at 8: below it, r206 matches only 45%.
  - With this machine's hardware FYL2X for LN, 3 points miss. At those, RN64 of the log lands exactly on a
    53-bit tie, where the exact LN model gives Excel's value, so the kernel takes Excel's LN unchanged.
- **GAMMALN for 3 <= x < 8 (W2, structure).** Excel reduces t into [2,3) and forms the ascending stored product
  p = t*(t+1)*...; then GAMMALN = fl(G(t) + LN(p)), with G a base kernel on [2,3) that is still open. With
  G replaced by the exact value it reaches 91-94% exact; the product order matters.
- **GAMMALN for small x is an exact one-step recursion (W1, verified by the coordinator).**
  GAMMALN(x) = fl53(fl64(GAMMALN(fl(x+1)) - LN(x))), with Excel's own GAMMALN and LN.
  - Probe `kmyst-gam-w101` holds B = GAMMALN.PRECISE(x), C = LN(x) and E = GAMMALN.PRECISE(x+1) side by side.
    The identity holds on 12,288 of 12,288 points at x0 = 0.0213, 0.1517, 0.2911, 0.4123, 0.5037 and 0.6129.
  - The switch is the exact comparison x < 0.7, with 0.7 the double 0x1.6666666666666p-1. From `w103`, the
    LN-error slope is -1.00 on the 256 doubles below and +0.12 from 0.7 up. Tiny x is the same formula, since
    fl(1+x) = 1 and GAMMALN(1) = 0, so no separate eps branch is visible.
  - So everything below about 0.7 reduces exactly to the near-1 kernel on [0.7, 1.7).
  - The near-1 piece on [~0.7, 1.5] is fl(t*A) with t = x-1 exact and A a stored double. It uses no LN, and its
    structure is open (W1).
- **GAMMA for x >= 10 is 99.84% exact (W4, verified by the coordinator).** Every op is fl53(fl64(.)), with
  EXP model V6 and LN = FYL2X:
  - h = x/2; n = floor(h); c2 = h - n; r = x^n by right-to-left binary powering, every square and multiply stored;
  - v = r*EXP(c2*LN(x)), which is x^(x/2);
  - w1 = 1/x; s = the 10-term Bernoulli series of large-x GAMMALN, in w1; W = EXP(s);
  - t = EXP(-x)*sqrt(w1); t = t*v; t = t*v; t = t*2.50662827463100050242 (sqrt(2pi)); GAMMA = t*W.
  - Score: 155,714 of 155,964 committed points with 10 <= x < 171.6243769563027, excluding integers <= 88 and
    exact half-integers (`gamma_big_check.c`). Integers 89-171 match 85 of 85.
  - Each of the 250 misses is fixed by flipping one stage rounding. 80% are W = EXP(s) near-midpoints
    (0.003-0.014 ulp), so Excel's s differs by about 1 ulp of s near x = 10-16. Using Excel's LN column instead
    of FYL2X does not change the rate.
  - Quirk: the binary powering, with v used twice, makes Excel's GAMMA up to about +-40 ulp off the true value
    near x = 150.
  - Special paths: integers n <= 88 take the downward product (n-1)(n-2)...2. Exact half-integers m+0.5 >= 10.5
    take another, more accurate path (unsolved, W3). The cut is exactly x >= 10.
- **GAMMA at exact positive half-integers is bit-exact (W3, verified by the coordinator).** Exact equality
  x == floor(x) + 0.5 triggers it: p = 1; for k = 0.5, 1.5, ..., x-1 ascending, p = fl(p*k); then
  GAMMA = fl(p * 0x1.c5bf891b4ef6bp+0), sqrt(pi) as a double. That is 172 of 172 points, x = 0.5 to 171.5
  (`kmyst-gam-h101` and t1042-gamma-half-int). The neighbours at +-1 ulp take the large-x path. Descending
  order, sqrt(pi) first, or a 64-bit product all fail (33-58 of 172). Negative half-integers are open.
- **GAMMA at negative half-integers is bit-exact (W3; the coordinator verified the magnitudes).**
  q = -x; GAMMA = +-fl(fl(PI/Gh(q))/q), with Gh the positive half-integer model and PI = pi as a double. That is
  171 of 171 nonzero points, -0.5 to -170.5 (`kmyst-gam-h102`). x <= -171.5 gives 0 (flush below DBL_MIN).
- **GAMMA reflection for x <= -30 (W3): 6,143 of 6,144.**
  - q = -x; p = floor(q); z = q - p; if z > 0.5 then p += 1 and z = q - p;
  - s = |RN53(FSIN(RN53(PI*z)))|; GAMMA = +-fl(fl(PI/G(q)) / fl(q*s)), with G(q) = Excel's GAMMA(q).
  - The one miss, x = -0x1.94efe288f63adp+5, is an FSIN tie: RN64(sin) lands on a 53-bit midpoint, and
    RN53(sin) misses a different point. Engine form: s = RN53(RN64(sin(RN53(PI*z)))).
  - Checker: `gamma_neg_check.py` (half-integers of both signs, plus the reflection; signs checked).
  - Small |x| (-9.87, -2.25, -1.73, -0.31) does not reflect; it presumably recurs up to G on [1,2). `h104`
    locates the cut.
- **GAMMA for 0 < x < 1 is fl(fl(1/x) * G(fl(x+1))) (W4, verified by the coordinator).** On `kmyst-gam-u101/u102`,
  D = fl(fl(1/x)*E), with E = Excel's GAMMA(x+1), holds on 18,431 of 18,432 points.
  - The one miss is x = 0.5+1ulp. There fl(x+1) = 1.5 exactly, so the E column takes the half-integer special path,
    while GAMMA(x) itself uses G(1.5). It is not a model miss.
  - (-1,0) does not follow this identity (8,197 of 14,336). It uses an upward division chain instead (W3, verified):
    z = fl(1/x); y1 = fl(x+1); z = fl(z/y1); y = fl(y1+1); GAMMA = fl(z*G(y)).
    That is 14,080 of 14,080 points where fl(fl(x+1)+1) = fl(x+2), with G read from the GAMMA(x+2) column. The
    general form for -10 < x < 0 is probably z = 1/x; y = x+1; while y < 1 { z = z/y; y = y+1 }; GAMMA = z*G(y).
    Confirmed on `kmyst-gam-h105` (runs at -1.41, -2.25, -5.37 and -9.87): z = fl(1/x); y = fl(x+1); while y < 1 then
    z = fl(z/y) and y = fl(y+1); GAMMA = fl(z*G(y)). That is 4,096 of 4,096, with G read from GAMMA at the chained
    argument (the worksheet sum x+1+...+1 equals the chain's y on every row).
- **GAMMA reflection cut (W3): exactly -x > 10**, mirroring the positive cut at x >= 10.
  - `kmyst-gam-h104` gives 112 of 112 on the sweep from -10.04 to -34.19.
  - -10-1ulp reflects; -10+1ulp does not; -10 itself is #NUM!.
- **GAMMA for 1 <= x < 10 reduces to [1,2), not Cephes's [2,3) (W4, verified by the coordinator).**
  y = x; z = 1; while (y >= 2) { y = y - 1; z = fl(z*y); }  GAMMA = fl(z * G(y)), with G = Excel's GAMMA on [1,2).
  The product runs descending, (x-1)(x-2)..., each step stored.
  - Evidence, `kmyst-gam-k101..k104` (GAMMA of x, x+1, x-1, x+2, x+3 on 51 runs of [2,3)), with Excel's own G in
    column F:
    - GAMMA(x) = fl((x-1)*G): 52,224 of 52,224.
    - GAMMA(x+1) = fl(fl(x*(x-1))*G): 52,224 of 52,224.
    - x+2 and x+3 hold the same way on the exact rows.
  - Integers fall out as the downward product, since G(1) = 1.
  - G on [1,2) is the remaining GAMMA kernel. G(1+2^-52) = 1-2^-53 is correctly rounded, which Cephes's 1/y chain
    does not give.
- **GAMMA (W4, superseded structure note).** It follows Cephes `gamma`: the downward product while x >= 3, division while x < 2,
  then fl(fl(z*P(t))/Q(t)) on [2,3). It is bit-exact at x near 1.37e-9. Excel's P/Q coefficients are its own,
  running 5-9e-17 above Gamma.
- Helpers: `rnd.py` (exact RN53/RN64), `exp_v6.py` (Excel EXP, T-1778).

## GAMMALN base kernel on (2,3]: state at W2's final report
- y = fl((x-2)*S), with S a stored double; x = 3 belongs to this piece.
- Exact pins at x = 2 + 2^j*2^-51 give S minus lnGamma/(x-2) = -0.67, -0.25, +0.59, -3.73, +1.64, +1.36, -1.19,
  -1.30, -3.50, +1.09, +0.26, +0.61 ulps (j = 0..11).
- S-deviation lines at x = 2 + k*2^-51, pure sine with phase 0 at k = 0, in cycles per step (weight in ulps of S):
  0.0213747 (+1.69), 0.0505811 (+1.69), 0.1889601 (+1.67), 0.1808971 (+1.65), 0.2063614 (-1.47),
  0.1938142 (-1.42), 0.0732403 (-1.41), 0.4078820 (+0.83), 0.4058033 (-0.74).
  This points to Horner in a variable that is dyadic at 2 (raw x, x-1 or x-3), not m = x-2 alone.
- **Negative results.** Relative-minimax rationals of lnGamma/(x-2), also divided by (x-1), (x+1) or x, all failed.
  None explains more than 4 lines.
  - variables x-c for c in {0, .5, 1, 1.5, 2, 2.5, 3, -1};
  - 8 intervals, plus 1-D interval families;
  - degrees (4..10, 4..11);
  - normalisations lead = 1, const = 1, both monic, free lambda.
  Also out: Cephes lgam B/C, Boost (z-2)(z+1)(Y+R), Cody P2/Q2, LN(Gamma) and log1p(Gamma-1).
- **Likely reason** (W1's near-1 finding, the same as T-1778 ERFC [0.5,4]): Excel's coefficients sit far out
  along a near-null direction, a common smooth factor in N and D that cancels in N/D. No function or minimax fit
  can find them; per-stage phase ladders (epsilon ladders at a dyadic point) can.
- 1.5 < x < 2 is a different composition, not cracked.

## GAMMALN near-1 kernel, 0.7 <= x <= 1.5: state at W1's report
- res = fl(t*A), t = x-1 exact; A = fl(N(x)/D(x)), Horner in raw x, degree (9,9) or (9,10).
- Candidate: `near1_candidate_w1.txt` (49% exact on [0.7,1.5]). The stage values at x = 1 match Excel to 1-8 ulps.
- Method that worked: per-chain phase ladders near x = 1 (`kmyst-gam-e1xx`, `e2xx`). Each step's Taylor terms
  follow a_k(j) = a_k(j-1) + a_k-1(j-1), so all stages of a chain are unwrapped jointly, rung by rung.
- A sharper function target than true lgamma: per-run mean of (Excel A minus exactly simulated model A).
- Remaining: an exact integer search on the stage mantissas at x = 1 (possibly with q0 = 2^-13 fixed), then the top
  coefficients from far runs.

## GAMMALN near-1 kernel: W1 final state
- Structure is established, confirmed by the line spectrum near 1 and by the within-run noise profile. Excel and the
  model agree on 98 far runs to within 0.044 ulp rms.
- N has degree 9 ((8,9) fails at chi2 8e4/pt). D of degree 8, 9 or 10 fits equally; q0 = 2^-13 is supported.
- Best set: c5_f99t (`near1_candidate_w1.txt`, N3/D3). 51.0% exact on [0.7,1.5]; 32% far (32.1% of 381k points
  on `kmyst-gam-f1xx`); near-1 A 93.5%.
- The gap is the MID coefficients N2-N7 and D2-D7. A 1-ulp change in one of them flips 30-38% of far A values;
  37 ulps on N9 or D9 flips less than 0.1%.
- Validated as BLIND to mid-coefficient errors: per-stage phases far from 1, the 0.75 and 1.375 ladders (1.25 sees
  N4 only), and the noise-profile degree test. Greedy and SVD-direction searches plateau at about 51%.
- Next idea: a joint integer search over about 12 mid coefficients, restricted to moves that keep the near-1 stage
  values, scored by exact match on far data.

## Zero-factor structure: withdrawn (coordinator)
- `roots.py` showed numerator roots at the other lgamma zero:
  - x = 2 for the near-1 N, with N3(2) = 4.3e-12;
  - x = 0.99999999 for the base N.
- `cody_cmp.py` compares Cody's real ALGAMA tables, `www.netlib.org/specfun/algama`, and shows the same thing.
  - Near 1, Cody uses D1 + t*P7/Q8, and its value at x = 2 is -8.9e-13. On (1.5,4) it uses D2 + t*P7/Q8, and its
    value at x = 1 is 4.7e-12.
  - That comes from a good rational tracking lgamma's analytic continuation just outside its interval. It is not
    an exact factor.
- So N(2) = 0 and N(1) = 0 are not valid hard constraints. The workers were told the same day.
- The poles differ from Cody's, so Excel ran its own fits:
  - near 1, Cody -46.07, -8.196, -2.952, -1.306, -0.5915, -0.2593, -0.0931, -0.01655;
  - near 1, Excel about -27.0, -7.45, -3.108, -1.476, -0.723, -0.320, -0.1165, -0.0209;
  - (1.5,4): Cody -129.0, -23.03, -8.339, -3.735, -1.762, -0.799, -0.272, -0.0479;
  - (2,3] base: Excel -20.06, -3.526, -1.201, -0.540, -0.280, -0.132, -0.0369.
- `factor_test*.py` (factored evaluation orders) are kept as a negative.
- Literature pass:
  - Cody and Hillstrom (Math. Comp. 21, 1967) give minimax rationals on [0.5,1.5], [1.5,4] and [4,12].
  - The Unix V6 gamma.s uses Hart and Cheney #5243 (near zero) and #5404 (asymptotic), with a Stirling cut at 8,
    as Excel's GAMMALN has.
  - No published source documents Excel's GAMMALN.PRECISE algorithm.

## GAMMA's G on [1,2): structure solved (W4)
- G(y) = fl(N(y)/D(y)). N and D are independent raw-y Horner chains of degree (8,8), every op fl53(fl64).
- It is Cody's GAMMA shape, (xnum+xden)/xden in z = y-1, re-expanded into raw y, with Excel's own coefficients.
  Cody verbatim scores 34%.
- Ruled out by the lines: 1 + X/D, (X+D)/D as evaluated, z-form chains, and Boost's Lanczos.
- **Triangle-law correction (applies to every kernel).** A product rounded into a sum that lands one binade up
  (a "triangle" line) has local frequency F(y) = S'(y)/2^(eS+1), where S = M*y + c is the sum. It is not the
  product's (M + tM')/2^e, and not u/2. With this law, every strong line matches its full track on an untuned fit.
- Line to stage map (mean/max track deviation):

  | Line | Stage | Kind | Mean / max deviation | Track |
  |---|---|---|---|---|
  | L1 0.40945 | D*c0 | product | 0.0005 / 0.0012 | to 1.47 |
  | a1099 0.10985 | D*c1 product + N*c0 | doublet (n0 ≈ d1, from Gamma's residue at 0) | | |
  | L2 0.42301 | final N sum | triangle | 0.0007 / 0.0011 | to 1.44 |
  | L3 0.12506 | M_Dc0 sum | triangle | 0.0007 / 0.0009 | to 1.995 |
  | a3048 0.30480 | M_Dc1 sum | triangle | 0.0008 / 0.0010 | |
  | 0.29108 | M_Nc0 sum | triangle | 0.0013 / 0.0043 | |
- Pins:
  - M_Dc0(1) = 1.4094486825857*2^16; M_Dc0'(1) = 0.2501178295786*2^16;
  - M_Dc1'(1) = -0.6096059494094*2^15;
  - N'(1) = 0.4230053820552*2^17.
- Candidate `g_candidate_w4.txt`: 29.9% exact (W4). The coordinator's `g_check.py` reproduces 28.5% exact and
  69.6% within 1 ulp on 871,392 points. The rate falls from 33% near 1 to 22% near 2.
  - Six soft directions are unpinned by the lines, and the coordinate search stalls at about 30%.
  - Next: a per-stage complex-amplitude phase solve against exact simulation, on the e301-e324 ladder plus the
    dense y101..y116 runs.

## Near-1 common-factor hypothesis: negative (W5, `w5/README.md`)
- c5_f99t scores 47.25% on all 980,655 points in [0.7,1.5).
- The doublet form (8,8)*(1+a*x) is at random-smooth level across 6,003 values of a. c5 has no shared N/D root:
  its spare pair is a lone N zero at -353 with D9 ≈ 0, a (9,8) shape.
- Positive control: the exact-score basin in a is only about 1e-14 relative wide, so blind scans of a common factor
  cannot find it.
- c5 is a sharp local optimum along all four function-preserving SVD directions.
- New tool, the x=0 Taylor invariant of N/D (a common factor cancels): R0..R3 = -5.40585248391, 134.942282379,
  -5937.32019206, 281011.5087. Relative and absolute minimax families of lgamma/(x-1) and lgamma/((x-1)(x-2)),
  degrees (7,7)-(9,9), are excluded at residuals of 1e-5 to 1e-3.
- Next (W5): re-read the near-1 lines with W4's triangle law and run a per-stage phase solve.
- **Update (W4 multi-anchor solve): G is degree (7,7), not (8,8).** In the (8,8) solutions both top coefficients
  collapsed to about 1e-8.
  - Fixing N8 = D8 = 0 gives Cody's coefficient count. The joint phase solve on anchors 1, 1.25 and 1.5, plus an
    exact search along the invisible SVD directions, then reaches 73.4% exact (W4).
  - D0 = -1/16 within 7e-10; rounding it to -1/16 changes nothing.
  - Candidate `g_candidate_w4_77.txt` (fit77S; W4 first sent it labelled fit77S2): 71.6% exact and 87.1% within 1 ulp on 1,448,928 clean GAMMA(x)
    points (`g_rows.py`). By tenth of [1,2): 92 85 78 69 66 66 66 52 48 73%.
  - An earlier 76.4% used an extractor that also picked up non-GAMMA(x) cells; it is withdrawn.
  - The best sets fit Gamma only to 1.2-1.5e-16, so Excel's G is itself about 1-ulp accurate.
- Anchor ladders: e701-e718 at y = 1.25, 1.5 and 1.75.
- fit77cS (`g_candidate_w4_77cS.txt`): 73.5% on W4's G.txt, and 72.3% exact (86.3% within 1 ulp) on the 1,448,928
  clean points. By tenth: 90 83 76 73 78 78 67 62 55 50%. fit77S is better above 1.9 (73% vs 50%); that tenth is dominated by the e3 ladder runs ending at 2 - c*2^-j, which the joint solve had not used (anchor 2).
  - y-bias of the misses (W4): model high near 1.0-1.1 and 1.86-2.0 (worst 1.90-1.92, 32% exact); model low across
    1.12-1.6.
  - Mean (Excel - model) is about 0.16 ulp rms, smooth in y.
  - No single stage is blamed: the remaining gap is in the weakly visible higher-stage sums (N c2..c6, D c3..c6).
  - Long 8,192-point ladders at 1.125, 1.375, 1.625 and 1.875 (e1001..e1025) are queued to lift those lines.
- **AN2 (`g_candidate_w4_AN2.txt`)**, W4's stochastic hill-climb on combined data: 78.2% exact and 89.3% within 1 ulp
  on the 1,448,928 clean points (W4 measures 76.6% on its 1,250,870).
  - By tenth: 93 80 80 72 71 71 70 75 47 79%. The weak spot is y = 1.8-1.9.
  - Linearised steps from the five-anchor phase solve all lower the exact score, so exact-match search is now the
    working tool.

## Excel's G is biased high (coordinator, model-free)
- `g_bias.py`: the mean relative error of Excel's GAMMA against true Gamma on [1,2) is +6.9e-17 (+0.62 ulp) near 1,
  rising smoothly to +1.14e-16 (+1.0 ulp) near 2 (60k-point sample).
- `g_eqosc.py`: AN2's error is likewise one-signed (+4.5e-17 to +1.2e-16) and does not equioscillate.
- So Excel's G is not a minimax-accurate rational. Its function error is a smooth ~1e-16 curve, plausibly
  coefficient rounding or limited-precision generation. See HANDOFF.md section 6.
- `HANDOFF.md` summarises the whole task for a fresh reviewer.
