# T-1941 handoff: Excel's GAMMALN, GAMMA and LN kernels (state at 2026-10-19 09:10 UTC)

This handoff is for a fresh reviewer or brainstormer. It sets out what is solved, what is open, what has been
tried, what failed and why, and where the evidence lives. Read this file first. Then read `README.md` in this
directory (the running log), `docs/research/excel-kernel-playbook.md` (the method from T-1778, ERF/ERFC) and the
worker write-ups in `w1/`, `w4/G_lines.md`, `w5/`, `w5/lines/` and `w5/p3/`.

- **Branch:** `agent/research/T-1941-crack-gammaln-gamma-ln` (research only; never force-push).
- **Task:** `docs/tasks/M9/T-1941-crack-excel-s-gammaln-gamma-and-ln-kernels.md`.
- **Rules:**
  - Excel is the spec.
  - Do not disassemble Excel.
  - Never print or commit RUSTSHEET_ORACLE_* values.
  - Commit messages are "T-1941: ...".

## 1. The goal and why it is hard
- Match Excel's GAMMALN.PRECISE, GAMMA and LN to the last bit, using oracle data only.
- Excel computes on the x87 FPU, so every op is double-rounded: fl53(fl64(.)) per stored op.
- The kernels are private rational approximations with Excel's own coefficients.
- ERF/ERFC were cracked this way in T-1778, and the playbook distils that.

## 2. Solved, bit-exact or essentially so (verified by the coordinator on committed data)
| Piece | Model | Score |
|---|---|---|
| LN(x) | RN53(RN64(RN64(ln2) * log2 x)), i.e. FLDLN2 + FYL2X | 114,687 / 114,688 (1 FYL2X near-tie) |
| GAMMALN, x >= 8 | 10-term Bernoulli Stirling, every op stored (README) | 291,957 / 291,957 |
| GAMMALN, x < 0.7 | fl(GAMMALN(fl(x+1)) - LN(x)), exact switch at x < 0x1.6666666666666p-1 | 12,288 / 12,288 |
| GAMMALN, 3 <= x < 8 | fl(Gbase(t) + LN(p)), t in [2,3), p the ascending stored product | structure; Gbase open |
| GAMMA, x >= 10 | binary-powering x^(x/2) form with Excel EXP/LN (README) | 155,714 / 155,964 (99.84%) |
| GAMMA, 1 <= x < 10 | descending stored product times G(y), y in [1,2) | 52,224 / 52,224 given G |
| GAMMA, 0 < x < 1 | fl(fl(1/x) * G(fl(x+1))) | 18,431 / 18,432 |
| GAMMA, -10 < x < 0 | z = 1/x; y = x+1; while y < 1 { z = z/y; y = y+1 }; z*G(y) | 18,176 / 18,176 given G |
| GAMMA, x <= -10 | reflection +-fl(fl(pi/GAMMA(q))/fl(q*s)), s = abs(RN53(FSIN(RN53(pi*z)))) | 6,143 / 6,144 (1 FSIN tie) |
| GAMMA at half-integers | ascending product times sqrt(pi) double; negatives via pi/(Gh*q) | 172/172, 171/171 |
| GAMMA at integers <= 88 | downward product | exact |

So everything reduces to **three open kernels**:
- **K1:** GAMMA's G on [1,2).
- **K2:** GAMMALN's base on (2,3], plus the different composition on (1.5,2].
- **K3:** GAMMALN's near-1 piece on [0.7,1.5).

## 3. The open kernels: state of play

### K1: GAMMA G(y), 1 <= y < 2 (W4). Best: 78.2% exact, 89.3% within 1 ulp
- **Structure (solid):** G = fl(N(y)/D(y)), with N and D independent raw-y Horner chains of degree **(7,7)**
  (8 coefficients each), every op stored.
  - It is Cody's GAMMA shape (xnum+xden)/xden, re-expanded into raw y, with Excel's own coefficients.
  - **D0 = -1/16 exactly** (fitted within 7e-10).
- Every strong rounding line in the data is assigned to a stage and validated over its whole track
  (`w4/G_lines.md`). The last two D products and several sums are pinned to about 1e-13 relative.
- **Best set:** `g_candidate_w4_AN2.txt`. Found by a stochastic hill-climb on exact matches over combined data.
  - By tenth of [1,2): 93 80 80 72 71 71 70 75 47 79%. The weak spot is y = 1.8-1.9.
- **Why it stalls:** the higher stages (N c2..c6, D c3..c6) are only constrained to 1e2-1e7 ulps by lines too
  weak to see. The multi-anchor phase solve is consistent (5 anchors), but every linearised step lowers the
  exact score, so exact-match search is currently the working tool.
- **NEW, model-free (coordinator, `g_bias.py`):** Excel's GAMMA on [1,2) is systematically HIGH against true
  Gamma.
  - The mean relative error is +6.9e-17 (+0.62 ulp) near 1, rising smoothly to +1.14e-16 (+1.0 ulp) near 2.
  - Correct rounding would average about 0.
  - AN2's own error against Gamma is likewise one-signed, +4.5e-17 to +1.2e-16, with 8 extrema and no sign
    change (`g_eqosc.py`). So Excel's G is **not** a minimax-accurate rational with tiny error.
  - Its function error is a smooth, one-sided ~1e-16 curve. That is the size you get from rounding the
    coefficients of an ill-conditioned rational to double, from a generator that ran in limited precision, or
    from a rounded scale factor applied to all coefficients.
  - **The curve is a fingerprint of how the coefficients were made, and nobody has exploited it yet.**
- **Pending data:**
  - e1001-e1025: long 8,192-point ladders at 1.125, 1.375, 1.625 and 1.875, filling now.
  - Already committed: y101-y116 (dense), e301-e324 (anchor 1, plus runs ending at 2), e701-e718 (anchors
    1.25, 1.5, 1.75).

### K2: GAMMALN base on (2,3] (W2). Best: about 76% exact overall, 93-94% on ladder clusters
- **Structure (W2's best reading):** y = fl((x-2)*fl(N(x)/D(x))), with N and D raw-x Horner, every op stored.
  The final op is clearly this (93.9% against <= 75% for 6 alternatives).
- **Degree:** (9,8) with a tiny x^9 term (1.5e-10 relative), or something close.
  - Forcing (8,8) plateaus at 32-42%. That is unlike K1, where dropping a degree jumped the score.
  - Unresolved: could be a basin problem.
- **Key diagnostic:**
  - The late stages (last 4 of each chain) are pinned by ladders, and need 0 offset at every anchor.
  - The misses come from **early-stage** values (N sums 1-5, D sums 1-4) being off by less than 1 ulp.
  - Tuned at a single anchor, the plain raw-x structure reaches **99.97%** exact (2.5 cluster), so the structure
    can be exact.
  - But no single coefficient set satisfies all anchors at once: each anchor alone reaches 98.6-99.97%, jointly
    2.236 drops to 83% and 2.718 to 76%, and the global score stays about 76%.
  - So either the parametrisation is wrong (degree, or the early-stage evaluation scheme) or the basin is wrong.
- Poles are within 0.2-5% of Cody's ALGAMA (1.5,4) poles: the same family, a different fit.
- W2's best (9,8) set "grid_a" (76.3% global) is not committed as a file. Its coefficients, HIGHEST degree
  first, are:
  - p = 0x1.5717994ef45d1p-51, 0x1.5e5b73810edd1p-16, 0x1.0ee2f118ad254p-9, 0x1.572bf8683b589p-5,
    0x1.09fd1ad1da445p-2, 0x1.f0a8c05397a9fp-2, -0x1.08305b47998b9p-6, -0x1.0ae34eb3cbad6p-1,
    -0x1.de852aee8c153p-3, -0x1.22260ed97843dp-6;
  - q = 0x1.03cadf2db0d9dp-18, 0x1.52fc9914ab713p-11, 0x1.5303658ef66c8p-6, 0x1.bb43b464d3259p-3,
    0x1.c515177eb1755p-1, 0x1.7b39fc9194438p+0, 0x1.eddf2040d2bc2p-1, 0x1.9b252b2fab347p-3,
    0x1.00000000000eap-7.
  - The constant 2^-7 is unidentifiable from data. Do not read it as structure.
- **Data:**
  - e401-e407 and e501-e512: ladders just above 2.
  - e601-e609: anchors 2.5 and 3-.
  - e801-e818: anchors 2.125, 2.25, 2.375, 2.625, 2.75, 2.875.
  - r-runs; allX long runs.
- **Also open:** (1.5,2] uses a different composition and is not cracked.

### K3: GAMMALN near-1 on [0.7,1.5) (W1, then W5). Best: 47.25% exact on all 980,655 points (c5_f99t)
- **Structure:** res = fl(t*A), t = x-1 exact, A = fl(N/D), raw-x Horner, every op stored.
  - N has degree 9 ((8,8) and (8,9) are infeasible).
  - D has degree 9 with D9 ≈ -2^-31, or degree 8 (undecided).
  - q0 = D0 = 2^-13.
- **Best set:** `near1_candidate_w1.txt` (set N3/D3 = c5_f99t). Near 1 it is 93.5% A-exact; far from 1, about 32%.
- **W5 (`w5/lines/README.md`):** with the triangle law, W1's "unexplained" far lines are tie lines of c5's own
  sum stages.
  - Round-half-even ties read coefficient bits directly: Excel's D3 is a multiple of 4 ulps, N2 is even and N5
    is odd.
  - Flipping D3 and N2 accordingly reproduces the line amplitudes.
- **W5 p3 (`w5/p3/README.md`):** degree reduction loses c5's near-1 basin. Exact search along the 16 softest
  SVD directions gives +0.4 points only.
  - A D-only tie-line lattice works on synthetic data, but on real data no D move within 1e7 ulps fits (rms 0.23
    against 0.035). Excel's D is far from c5 in stage terms, or structurally different.
- **Data:** e1xx and e2xx (near 1), f1xx (far runs), w1xx. e901-e915 (anchors 0.75, 0.875, 1.125, 1.25 and
  1.375) are queued behind e10xx.

## 4. Methods that worked (keep using)
1. **Composition first, with identity columns.** Probing GAMMA(x), GAMMA(x+1), ... and GAMMALN with LN side by
   side cracked every reduction and wrapper exactly.
2. **An exact x87 simulator per hypothesis.** Use numpy longdouble with astype(float64), or C with
   `-mfpmath=387` and volatile stores.
3. **Line spectra of (Excel - exact) over long consecutive-double runs, read with the RIGHT law:**
   - sawtooth (product) lines have local frequency (M + tM')/2^e;
   - **triangle lines** (a product rounded into a sum one binade up) follow S'(y)/2^(eS+1);
   - ties give lines that read coefficient parity.
   Reading triangles with the sawtooth law made stages look "invisible" for days.
4. **Full-track validation.** A stage hypothesis must match a line's frequency curve across the whole interval
   (dense runs at many y), not just at one point.
5. **Multi-anchor epsilon ladders** (runs at a ± c*2^-j, j = 52..6, irregular c), plus a joint phase solve
   against exact simulation: a constant column for triangles, and rung-by-rung unwrapping.
6. **Degree reduction when top coefficients collapse** (K1: 30% to 73% by going from (8,8) to (7,7)).
7. **Stage-offset probes** (W2): add integer ulp offsets to each stored partial sum per cluster to locate which
   stages carry the error.

## 5. Dead ends and traps (do not repeat)
- **Published coefficients:** Cody ALGAMA and GAMMA, Cephes, Boost (Lanczos and lgamma_small), Hart #5243 and
  Numerical Recipes are all wrong to the bit. Cody verbatim scores 34-36%. Use them for structure only; netlib
  is now reachable.
- **Exact-zero factor:** N(2) = 0 near 1, N(1) = 0 for the base. These are analytic continuation; Cody shows the
  same 1e-12 near-roots. Retracted.
- **Common-factor or doublet (1+a*x) parametrisation (W5):** negative, with a positive control. The basin is
  1e-14 wide.
- **Generator families (W5):** relative or absolute minimax of lgamma/(x-1) and lgamma/((x-1)(x-2)), degrees
  (7,7)-(9,9), many intervals. All excluded by the x=0 Taylor invariants of N/D.
- **Cody-style outer forms:** m*(D2 + m*N/D) (K2) and 1 + X/D (K1). Excluded because errors near the anchor would
  be suppressed, yet the lines there are full-amplitude.
- **Linearised joint steps from a far reference** lower the exact score: the near-null directions are not
  pinned. Use small steps or exact search.
- **Exact-match rate is misleading far from the truth.** Fitting to one dataset over-tunes: fit77cS scored
  73.5% on G.txt but lower on the full data. Always score on a broad, clean sample.
- **Coordinator mistakes caught:**
  - A G extractor picked up non-GAMMA(x) cells; fixed in `g_rows.py`, which keeps only cells whose formula is
    exactly `=GAMMA(A<row>)`.
  - Mislabelled candidate files.
  - Fill jobs chained with a self-matching `pgrep` wait loop silently never ran; start fills directly.
  - One overwrite of existing case names, reverted. Always check names are unused.

## 6. Ideas not yet tried (brainstorm seeds)
1. **Exploit K1's model-free bias curve.** Excel(G) - Gamma averages +0.62 to +1.0 ulp, smooth in y.
   - Hypothesis: Excel's coefficients are a high-precision fit rounded to double (or converted in limited
     precision), so Excel's function = R* + delta, where delta is the coefficient-rounding error.
   - Fit the bias curve jointly with the exact matches. A (7,7) R* of Gamma on [1,2] with D0 = -1/16 plus
     per-coefficient rounding offsets gives a small search. Try several targets: relative or absolute error;
     Gamma(y) versus Gamma(1+z); weighted fits.
   - Do the same bias measurement for K2 and K3 (`g_bias.py` is a template).
2. **Generator in low precision.** Test whether AN2 is a z-form (Cody-like) rational expanded to raw y in double
   or x87 arithmetic. Expansion rounding gives exactly a smooth ~1e-16 function error. Check the expansion of
   candidate z-form fits against AN2's coefficient low bits.
3. **Interval / MILP formulation** (as for small-x erf in T-1778).
   - Each Excel output bounds the final quotient to an interval.
   - For points where the late stages are pinned, propagate back to linear constraints on the early
     coefficients, and solve an LP/MILP over all anchors at once instead of exact-match search.
4. **Tie lines as a bit-reader across all kernels.** W5 showed that ties read coefficient parity and low bits
   (D3 = 0 mod 4 ulps). Systematically harvest ties for K1 and K2 too.
5. **K2 degree and scheme.** Test (8,7), (9,7) and (8,8) properly from fresh multi-anchor solves, not folded
   seeds. Distinguish "wrong early coefficients" (smooth per-point ulp corrections) from an Estrin or split
   scheme (rounding-noise-like corrections), using W2's pointwise early-stage solve.
6. **Cross-kernel consistency.** K1, K2 and K3 were likely made by one generator with one normalisation habit:
   - K1: D0 = -1/16;
   - K3: D0 = 2^-13;
   - K2: D constant 2^-7?
   A shared rule (for example "scale so D0 is a power of two in the raw variable") would constrain all three.
7. **Excel version history.** GAMMALN.PRECISE arrived in Excel 2010; legacy GAMMALN may differ and could expose
   an older, simpler kernel. Probe both side by side.
8. **More, longer ladders** where lines are weak: K1 at y = 1.8-1.9, K3 at far x. Oracle cost is about 2 minutes
   per 32,768-row case.

## 7. Tools (in this directory unless noted)
- **Data:**
  - `extract_runs.py OUT`: all kmyst-gam rows (x, B, C, D, case).
  - `g_rows.py OUT`: clean GAMMA(x) rows in [1,2).
- **Scorers:**
  - `g_check.py N D rows`: exact x87 scorer for K1, with per-tenth rates.
  - `w1/simA.c`, `w1/csim.py`, `w1/exsearch.py`: K3.
  - `gammaln_big_check.c`, `gamma_big_check.c`, `gamma_neg_check.py`, `small.py`, `lnmodels.py`: the solved
    pieces.
- **Line analysis:**
  - `w4/gline_tools/`: `vspec2.py`, `track2.py`, `stagephase4.py`, `gsim.c`, `simres.c`, `dense.py`.
  - `w5/lines/`: tie lines and a lattice.
  - `w5/`: x=0 invariants and generator scans.
- **Misc:** `roots.py`, `cody_cmp.py`, `g_eqosc.py`, `g_bias.py`; probe generators `gen_g19.py` to `gen_g22.py`.
- **Oracle:** `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Cases go in
  `eval/suites/probes-numeric/cases/kmyst-gam-*`; commit them with `git add -f`.

## 8. Workers at handoff time
- **W2** (session_01MzPmWSu7ifkc3av8wMNHtP): K2, a joint early-stage search over 10 anchors from (9,8) starts.
- **W4** (session_01RWLT45Yogi225NBsryQvQ5): K1, simulated annealing and hill-climbs from AN2. Waiting for
  e1001-e1025.
- **W5** (session_01AJhNGX8uNdacKiJaHWDVPH): stopped. Its suggested next step for K3 is a joint N+D phase
  solve with function rows on the e9xx anchors.
- **W1 and W3:** finished.

## 9. Reviewer critique and new plan (Fable 5.1)

State at 2026-10-19 10:30 UTC. Scripts and result tables are under `review/` (see `review/README.md`); every
number below was computed on the committed data (`g_rows.py`, `review/ln_rows.py`), in minutes, with exact x87
simulation (numpy longdouble with `astype(float64)` after every op).

### 9.1 Headline
1. **The model-free "bias curves" of K2 and K3 are staircases, not functions** (`review/bias_K2.txt`,
   `review/bias_K3.txt`). Excel's GAMMALN.PRECISE minus lnGamma, relative, averaged per 0.01 bin (SE about
   0.03 ulp), is piecewise constant with sharp steps:
   - K2 on (2,3]: -0.47 ulp on (2, 2.28); -1.45 on (2.285, 2.35); -2.3 on (2.355, 2.40); -1.15 on (2.405, 2.455);
     -0.07 on (2.465, 2.61); +1.3 on (2.615, 2.63); +0.7 on (2.635, 2.66); -0.33 on (2.665, 2.71); +0.05..+0.12 on
     (2.715, 2.78); +1.0 falling to +0.57 on (2.785, 3].
   - K3 on [0.7,1.5): 0 on (0.70, 0.73); -0.5 on (0.736, 0.752); -0.04 on (0.76, 0.82); +1.4 on (0.83, 0.87);
     +0.75 on (0.87, 0.91); +0.63 to +0.52 on (0.91, 1.065); -0.55 on (1.07, 1.105); -1.5 to -2.5 on (1.11, 1.16);
     -1.8 on (1.165, 1.19); -1.3 to -1.1 on (1.19, 1.5).
   - **Every step sits on a binade crossing of a Horner intermediate of the current candidate** (grid_a for K2,
     c5_f99t for K3; `review/k2_plateau_sim.py`, `review/k3_crossings.py`). K2: 2.2767 (N.P9/S9), 2.3407/2.3490
     (N.P8/S8), 2.394-2.410 (D.S7/P7, N.P7), 2.4634 (N.S7), 2.606 (N.P9/S9 again, N.P5), 2.631-2.658 (D.S6, N.P4,
     N.P6, N.S6), 2.707-2.724 (D.P6, D.S8/P8), 2.774-2.779 (N.P8, D.P3, N.S8). K3: 0.7303 (N.S8), 0.7513/0.7569
     (N.P8, D.S9), 0.820-0.829 (N.S9/P9, D.S7, res), 0.8619 (D.P7), 0.9028 (N.P5), 1.068 (N.S9/P9), 1.107-1.136
     (D.S8/P8, N.S8, N.P4, N.P2, res, N.P8), 1.159-1.162 (N.S5, D.S7, D.P4), 1.185 (D.P7). Crossing positions are
     insensitive to coefficient errors, so they are Excel's crossings too.
   - The exact simulation of grid_a's own chain (sim minus grid_a's exact rational) shows the same staircase at the
     same x, with different plateau heights (`review/k2_plateau_sim.py` output in `review/README.md`).
   - **Mechanism.** A stored sum S = fl(P + c) with P a stored product: when P lies on S's grid (same binade, or
     cancellation), S = P + RN_u(c) with u = ulp(S), so the stage carries the constant error RN_u(c) - c, up to
     +-u/2. When P is one or two binades below S, the error takes 2 or 4 values depending on P's low bits, and its
     mean is still a nonzero constant. The constant changes only when S or P changes binade. The output bias is the
     sum over stages of (weight) x (that constant), i.e. a staircase. Product roundings, the division and the final
     product are zero-mean and contribute no step.
   - **Consequence: the plateau heights read the bits of each constant c_k that lie below its sum's ulp**, which is
     exactly the information that function fitting cannot see and that exact-match search cannot climb to (below).
     Nobody has used it. W5 saw the bumps (part 1 lead 6, part 2 section 4) and correctly called them rounding
     bias, but treated them as a nuisance.
2. **The per-anchor/joint paradox of K2 is explained, and is not evidence about degree or scheme.** W2's anchors
   2.125, 2.25, 2.375, 2.5, 2.625, 2.75, 2.875 and 3- sit on plateaus -0.49, -0.45, -2.31, -0.10, +1.30, +0.04,
   +0.88 and +0.57 ulp. A single-anchor fit absorbs the local plateau into "early-stage values off by less than
   1 ulp" (W2's words); the plateaus differ per anchor, so no smooth refit joins them, and the (9,8) tiny x^9 term
   (1.5e-10 relative) is the artefact the playbook warns about (section 3, "tiny top coefficient"). The joint
   solution is per-stage bit reading, not a search.
3. **Exact-match search is on a plateau, and the hill-climbs have been optimising it.** Calibration on K1
   (`review/k1_degree_modelfree.py`): a (7,7) raw-y rational fitted in mpmath to Excel's *measured* function
   (bin means of GAMMA vs Gamma, 91 bins, SE 0.015 ulp) reproduces that function at chi2/dof 1.42 (max 5.3 SE),
   agrees with AN2's function to 1.4e-17..3.2e-17, and yet its rounded coefficients score **28.3% exact** (72%
   within 1 ulp). AN2 scores 78.2% with its function 0.1-0.3 ulp *further* from Excel. So 50 of AN2's 78 points
   come from stage values that match Excel near the anchors, not from the function, and no small integer move can
   improve it: a ridge sweep (`review/k1_ridge_sweep.py`) shows every integer move that flattens AN2's residual
   (by 0.1-0.3 ulp) drops the exact rate to 45-67%. Exact match is the final verifier, never the search objective.

### 9.2 Critique, point by point
1. **The bias-curve interpretation in section 6 (idea 1) is wrong for K2 and K3.** The curves are rounding
   staircases (9.1.1), not generator fingerprints. For K1 the curve (+0.59 at 1.0 to +0.97 at 1.94, SE 0.015-0.04)
   is smooth enough that a (7,7) fits it at chi2/dof 1.4, so there the function really is about 1 ulp high; but
   K1's low stages also chop 1-2 bits (N0 at 2^-37 against ulp(N) = 2^-35 or 2^-36), so part of even that curve is
   plateau. Measure the staircase by exact simulation before reading any curve as "function".
2. **Degree claims from exact scores are not evidence** (K2 "(9,8) or close", K3 "N degree 9, D degree 9 with
   D9 ~ -2^-31"). The playbook says so (section 4), and 9.1.3 quantifies it: the right function scores 28%.
   The only degree test that worked today is continuous: fit the measured function. For K1 it decides cleanly
   ((6,6): chi2/dof 5.8e5; (7,7): 1.42; (8,8): 3.8). For K2 and K3 it fails for every degree (chi2/dof 237 to
   53,000, `review/k23_degree_modelfree.py`) **because of the staircase**, so the degree is undecided until the
   plateau contribution is modelled or subtracted (plan, W-A/W-B step 4). W5's D9 ~ -2^-31 and W2's tiny p9 are
   both "tiny top" artefacts until shown otherwise.
3. **K1's weak spot (1.80-1.97) is a multi-stage plateau regime, not one parity bit.** Errors there are +-1..+-3
   ulps, nearly white between consecutive doubles (lag-1 autocorrelation -0.03 to -0.14), with P(|e| > 1) = 0.37
   (`review/k1_weakspot_hist.py`), and no single +-1..+-4 ulp coefficient flip reduces the dispersion
   (`review/k1_flipscan.py`: best 0.468 exact from 0.433). Six of AN2's intermediates cross binades in
   1.70-1.89 (N.P1 1.7295, N.S3 1.7029, N.P5 1.7493, D.P4 1.7807, N.P2 1.7826, N.P4 1.8888) and several sums
   there sit two binades above their product, so the low 2 bits of several constants decide the pattern.
   The e1001-e1025 ladders at 1.875 sample one plateau; what is needed is runs across the steps (probe P1 below).
4. **No positive control for the K1 joint phase solve or the K2 joint early-stage search.** The playbook makes
   this mandatory. Both solves "lower the exact score", which is expected (9.1.3) and says nothing about whether
   they move toward Excel. Generate synthetic Excel-like data from a (7,7) set placed 1e5-1e6 ulps from AN2
   along the null directions, on the real y grid, and show the solve recovers it. Until then neither method's
   failure is diagnostic.
5. **"Every op stored" is validated only where it is cheap to see.** In the Stirling kernel the products are far
   below the sums, so stored vs 64-bit products is invisible there. For K1, AN2's scheme variants score
   stored 78.2%, product-64-bit 49.8%, all-64-bit 38.0% (`review/k1_scheme_bins.py`); that is consistent with
   stored but not decisive since AN2 was tuned under it. The staircases are the stronger evidence: an all-64-bit
   chain would chop 11 more bits and produce plateaus 2^-11 smaller. So sums are stored, with 53-bit constants
   rounded into them. Products: the triangle/tie lines (W4, W5) already settle it for the low stages.
6. **(1.5,2] is not "stored GAMMALN(fl(x+1)) minus LN(x)".** On 121,577 points, GAMMALN(x) + LN(x), computed in
   64 bits, is a 53-bit double only 7.4% of the time (`review/gl_15_2_glue.py`); Sterbenz makes E - LN(x) exact
   on this range, so a stored E is excluded. Composition variants through grid_a all score about 3%, but that
   test is blind because grid_a's S is not exact to 2^-60; it needs the probe in P2. (I first read the
   B + C vs lnGamma(x+1) agreement as "the argument is the exact 64-bit x+1"; that is a tautology for any accurate
   B, retracted.)
7. **Normalisation habit.** D0 can only be measured to about ulp(final sum)/2, i.e. 2e-10 relative for K1 and
   2e-10 for K3; "exactly -1/16" and "exactly 2^-13" are consistent but their low 30 bits are unobservable and
   irrelevant (they never reach a stored value). Across the solved ERF kernels the habit is not universal:
   small-x erf has Q0 = 0.98994509777494666, not a power of two; the [0.5,4] kernel has 1 and the tail 128.
   Do not constrain K2's D0 on this basis (`review/norm_survey.py`).
8. **A "short decimal literal" prior is dead on arrival.** The bit-exact T-1778 coefficients need 16-17
   significant digits to round-trip, like generic doubles (`review/erf_decimal_digits.py`). Excel's tables were
   not typed as 15-digit decimals.
9. **Data hygiene near K2's left end.** With grid_a, the (2.000, 2.005) bin has std 120 ulps of B
   (`review/k23_resid_bins.py`), against 0.7-1.3 elsewhere. Either grid_a's S is wrong by many ulps very close to
   2 (the e4xx/e5xx ladders) or those rows need the formula filter. Check before using them as anchors.
10. **Side channels through other functions are weak by construction**: GAMMA(x) on (0,1), GAMMALN below 0.7 and
    the [3,8) recurrence all compose on *stored* kernel values (verified at 100%), so they expose nothing the
    direct column does not. The two exceptions are the (1.5,2] glue (point 6) and the untested [3,8) glue,
    where an unstored 64-bit product would leak extra bits (probe P2, P3).

### 9.3 Ranked ideas (expected value per effort)
**I1. Plateau bit-reader (new; highest value, 1-2 days per kernel).**
- Hypothesis: Excel's output bias on each plateau equals sum_k w_k(x) b_k, with b_k = mean rounding error of
  stage k's sum in that binade configuration, a known function of (c_k mod ulp(S_k)) and of how many binades P_k
  lies below S_k; w_k(x) is the exact stage weight (x^(top-k)/D for an N stage, times -N/D for a D stage, times
  the outer factor).
- Decisive test (cheap): take the candidate, list its crossings, measure Excel's step height at each crossing where
  a single stage crosses (SE 0.01 ulp with all points; use 0.002-wide bins either side), compute the candidate's
  predicted step for every value of c_k mod u (2-4 candidates per stage), pick the one that matches, set those
  bits, re-simulate, and check that the residual staircase flattens. Then the exact score must jump on the
  plateaus covered. Positive control first: perturb a known set's low bits, confirm the reader recovers them.
- Evidence for: the steps already coincide with the crossings (9.1.1) and the simulated staircase has the same
  shape. Against: if the steps do not match any low-bit choice of the crossing stage, the scheme at that stage is
  not "stored product + stored sum".
- Data: committed (all K2 and K3 rows; K1 dense runs). Tools: `review/k23lib.py`, `review/k2_plateau_sim.py`.

**I2. Two-scale solve with the right objectives (replaces exact-match search).**
- Function scale: fit the rational to Excel's measured function per bin, with the candidate's simulated staircase
  subtracted (or, better, after I1 has fixed the plateau bits). Target precision 0.01 ulp. This fixes the ~6
  function-visible directions.
- Bit scale: plateau readings (I1) for sums, line/tie phases (W4, W5 tools) for products, as one integer lattice
  problem per chain, solved by LLL/Babai with a box of 1e9 ulps, validated on synthetic data. The playbook's step 4.
- Effort: 2-3 days per kernel. Success: exact rate above 95% on all committed data, misses explained by
  near-ties.

**I3. K2 restart from W2's per-anchor offsets, re-read as bits (0.5 day).** The stage offsets W2 found per anchor
are (Excel b_k - candidate b_k) on that plateau. Tabulate them against the anchor's binade configuration and solve
c_k mod u per stage; that is I1 using data already in hand.

**I4. Degree and scheme by continuous tests on staircase-corrected curves (0.5 day each).** After I1, refit
(8,8), (9,8), (8,9), (9,9) to the corrected K2 curve and (8,8), (9,8), (9,9) to K3; the chi2 decides as it did for
K1. Also compare stored vs unstored-product schemes by their predicted plateau heights.

**I5. Probes (coordinator): P1-P4 below.** Dense runs across K1's steps are the only new oracle data this plan needs.

**I6. Tie lines as parity readers for K1 and K2 (W5 method; 1 day).** Still valid; lower priority because plateaus
read the same bits at lower cost, but ties give the bit *at* the sum grid when the plateau is ambiguous (ties have
zero mean).

**I7. Interval/MILP.** Subsumed by I2; keep as the final polish for the last 1-2 coefficients per chain.

**I8. Legacy GAMMALN vs GAMMALN.PRECISE (one case, P4).** Low prior, near-zero cost.

### 9.4 Execution plan (parallel Opus workers)
Common rules: exact simulation per hypothesis; positive control before any reading is trusted; report plateau
tables and bit readings in a README under `docs/research/t1941/<worker>/`; score only as the last step.

**W-A: K2 plateau reader.** Inputs: `review/ln_rows.py` dump, grid_a (`k23lib.py`), `review/bias_K2.txt`,
`review/k2_plateau_sim.py`. Steps: (1) positive control: flip 1-3 low bits of three constants of grid_a, generate
synthetic Excel on the real x grid, recover them from the synthetic staircase; (2) measure Excel's step heights at
each single-stage crossing with 0.002 bins and all points; (3) solve c_k mod u for every N and D sum stage that
crosses in (2,3]; (4) with those bits fixed, refit the function-visible directions to the corrected curve (I4) and
decide the degree; (5) score. Output: bit table, corrected candidate, degree verdict. Success: a single set above
90% exact on every anchor cluster at once, or a documented stage whose steps no stored-sum model reproduces.

**W-B: K3 plateau reader.** Same as W-A with c5_f99t and `review/bias_K3.txt` (plus W5's tie-bit readings as
cross-checks: D3 = 0 mod 4 ulps, N2 even, N5 odd). Treat the N.S5 zero crossing at about 1.485 (sign change,
huge relative noise) as a separate regime; exclude (1.47,1.5) from readings. Success as W-A.

**W-C: K1 two-scale solve.** Inputs: AN2, `g_rows.py`, W4's `gline_tools`, `review/k1_degree_modelfree.py`
(fits (7,7) to the measured function in mpmath), `review/k1_two_scale_lsq.py`. Steps: (1) positive control for
the joint solve (critique 4); (2) plateau map of AN2 on [1,2) and step readings at its 18 crossings (needs P1 for
1.70-1.95); (3) joint lattice with function rows (Excel target, not Gamma), plateau rows and the W4 line phases on
the dense runs; (4) score. Success: above 95% and the 1.80-1.97 dispersion gone.

**W-D: compositions and probes.** Runs P2-P4 when filled; tests the (1.5,2] and [3,8) glue with Excel's own
columns; writes the findings into `docs/excel-semantics/statistics.md`.

**Probe requests (exact layout; case names unused as of this handoff: `kmyst-gam-p1xx`..`p4xx`):**
- **P1 (K1 steps):** 26 cases `kmyst-gam-p101`..`p126`, 8,192 rows each: A = consecutive doubles starting at
  y0 = 1.70 + 0.0103137*i + 0.0031415 (i = 0..25, irregular starts, so about every 0.01 across 1.70-1.96),
  D = GAMMA(A). That gives both plateau heights and the W4 line phases across the K1 steps. Optional: 8 more
  cases straddling each listed crossing at +-200 ulps (4,096 rows, two per crossing) for sharp step heights.
- **P2 ((1.5,2] glue):** 4 cases `kmyst-gam-p201`..`p204`, 16,384 rows: A = consecutive doubles from 1.52, 1.66,
  1.81, 1.97; B = GAMMALN.PRECISE(A), C = LN(A), E = GAMMALN.PRECISE(A+1), F = GAMMALN.PRECISE(A-1), G = LN(A-1),
  H = GAMMA(A). Tests: B == fl(E - C) (already excluded on existing data, confirm), B == fl(F + G) (downward
  recurrence), B == fl(LN(H)) (log of GAMMA), and whether B + C carries bits beyond E.
- **P3 ([3,8) glue):** 2 cases `kmyst-gam-p301`..`p302`, 16,384 rows: A from 3.37 and 5.71; B = GAMMALN.PRECISE(A),
  E = GAMMALN.PRECISE(A-1) (for 3.37) / GAMMALN.PRECISE(A-3) (for 5.71), F = LN of the stored ascending product
  built in cells: G = A-1, H = G*(G+1), I = H*(G+2) (for 5.71), F = LN(G) or LN(I). Test B == fl(E + F) exactly;
  a miss rate above the K2 miss rate means the glue reads an unstored product.
- **P4 (legacy GAMMALN):** 1 case `kmyst-gam-p401`, 16,384 rows: A from 2.33 (consecutive), B = GAMMALN.PRECISE(A),
  C = GAMMALN(A), D = GAMMA.DIST(1, A, 1, FALSE), E = EXP(-B). Tests: B == C; D vs e^-1/GAMMA(A) composition.

### 9.5 What is settled by this review
- K2 and K3 "bias" is a rounding staircase on the candidates' own binade crossings; the per-anchor paradox follows.
- K1 is (7,7) by a continuous test; its function is about +0.6..+1.0 ulp high and smooth; AN2 is at a search plateau.
- (1.5,2] is not a stored-value subtraction; the short-decimal prior and the universal power-of-two D0 are out.

## 10. Execution of the section 9 plan (coordinator, 2026-10-19 11:10 UTC)
- **W-C (K1): frozen at AN2.** See `wc/README.md`.
  - The plateau reader does not apply to K1: its sums sit 1-2 binades above their products.
  - The joint solve converges on synthetic data only with measured-function rows plus a ridge, and stalls 1-4
    units out in the 6 weakest directions.
  - Exact-match is a needle on a ~75% plateau, about 1e6 ulps wide.
  - Adding all ~27 stage lines makes it worse.
  - Future route: per-stage law validation (`wc/README.md` section 5).
- **W-B (K3): plateau reader validated and applied.** See `w5/p4/README.md`; best set `w5/p4/ss_fc.txt`.
  - Six constants read; degree (9,9).
  - Staircase chi2 down 97%.
  - Exact 46.8% (coordinator), against c5's 46.0%.
  - Next: the product-level phase lattice with the read constants fixed.
- **W-A (K2): in progress** (W2).
- **W-D (compositions): pending** the P2-P4 fills.

## 11. Second review (Fable)

State at 2026-10-19 16:30 UTC. Scripts and result tables are under `review2/` (see `review2/README.md`). Every
number below was computed on committed oracle data with exact x87 simulation (`review2/gsim.c`, numpy longdouble),
in minutes. I did not run the oracle. Section 11.1 is the headline, 11.2 the critique, 11.3 the ranked ideas with
their tests, 11.4 the plan per worker, 11.5 the probe layouts.

### 11.1 Headline
1. **The x87 double-rounding model is now validated, not assumed.** `gammaln_big_check.c` recompiled with
   `typedef double L` (pure RN53 per op, SSE) scores 295,641 / 295,724 on x >= 8 against 295,720 for fl53(fl64):
   79 points separate the two, so Excel's stores really are double-rounded. Every simulator in this task is right to
   use fl53(fl64) at every op (and a candidate that only matches under pure RN53 would be wrong).
2. **K1: the "non-identifiable" verdict is wrong for the N chain's top, and the weak spot is a mid-stage problem.**
   An exact-template scan of the phases of the two invisible top products (`review2/top_bump.py`: nudge N product 2
   and 3 by d2, d3 in 1/16 steps, count exact matches, positive control = AN2 with N7 +3, N6 -5, N5 +2 ulps on the
   same grid, which gives bumps of +133 to +548 points at the right phases) finds on Excel's data a V centred at
   **(d2, d3) = (0, 0)** at the anchors 1, 1.5, 1.75, 1.875 and 2- (ranges 134 to 2,208 points) and on the 150
   dense runs at 1.70-1.96 together (range 2,615) and one by one (`review2/top_bump2.py`). A synthetic truth
   1e4 ulps away along the three most function-invisible directions (D0 fixed, function change 0.015 ulp) gives a
   flat collective profile (range 70 on 311k points) and random per-run phases, as the jitter argument predicts.
   So Excel's S_1(y) and S_2(y) equal AN2's as doubles on [1.70,1.96] and at the anchors: |dN7| < about 30 ulps,
   |dN6| < about 2 ulps, |dN5| < about 2 ulps. The 1.80-1.89 misses are not the top stages.
3. **Where one mid stage dominates the error, its phase is readable from one 8,192-point run of Excel's data**
   (`review2/stage_scan.py`, 1-D scans of all 14 product and 14 sum nudges per run): D product 5 (the phase of
   S_4^D(y0)*y0) gains +404, +498, +478, +343, +230, +609 points at phases -0.50, -0.50, +0.44, +0.31, +0.25, -0.31
   on p105, p108, p147, p149, p143, p122; N product 6 (phase of S_5^N) gains +232..+529 on p133, p134, p137, p138,
   p141, p145; N product 5 / sum 5 gain +312..+504 on p135, p136, p139, p140. These are modular readings of
   dS_4^D(y0) and dS_5^N(y0), the stage-value errors between anchors that the joint solve never reached (10.W-C).
   By contrast, on every run in 1.80-1.89 (p111-p119, p153, p154) and on all 19 rungs at 1.875 (e1019-e1025) no
   single-stage nudge gains more than 11 points while AN2 is 33-38% exact: there the error is a superposition of
   several stages' re-hashes (several intermediates cross binades in 1.70-1.89, 9.2.3), and no local coordinate
   has a gradient. That band must be solved after the single-stage runs have fixed the mid coefficients, not read.
4. **K2: W2's "law contradiction" (wa/README section 5) is reproduced by a few-ulp error in the UPSTREAM
   coefficients, and W2's control could not see it.** Synthetic truth = efC with p7 -2, p6 +3, q7 +2, q6 -3 ulps:
   a single-phase meter of N s5, N s4, D s4, D s3 at the anchors 2.125..2.875 reads +0.28/+0.28/+0.28/-0.22/-0.22,
   +0.25/+0.25/+0.25/-0.50/-0.50, +0.03/-0.47/-0.47/-0.47/-0.47, -0.47/-0.47/-0.09/-0.09/-0.22 with r = 0.07-0.39,
   while the true offset of those stages' own constants is 0 (`review2/k2_upstream_fm2.py`). A downstream-only
   truth (p2 +1, q1 -1) changes nothing (exact 1.000: those bits are below the sums' grids). W2's positive control
   moved p1 and q0, downstream of the metered stages, so it was blind to this failure mode. The re-hash matrix
   (`review2/rehash_matrix.py`) quantifies it: one ulp of p6 changes 34% of s4 and 23% of s5; p7 17% of s3; q6 50%
   of D s3 and 29% of s4; p5 changes 40% of the FINAL N. So the s4/s5 and D s3/s4 anomalies are what a 1-5 ulp
   error in p6/p7/q6/q7 (or their sub-grid bits) looks like through a single-phase meter. The stored-product law is
   not contradicted.
5. **Composition: nothing transfers between kernels.** Model-free reachability (W6's method extended to nested
   forms, `review2/xk_reach.py`, `review2/k2_inner_reach.py`): K3 = fl(t*A) (1.000 against base 0.50-0.99), (1.5,2]
   = fl(t*fl(m*R)) (1.000/0.89-0.96), K2 = fl(m*S) (1.000/0.52 at 2.7-2.8). K2's stored inner S is NOT fl((x-1)*R):
   where the test has power (2.10-2.26) S is reachable at exactly the base rate (0.886/0.890, 0.796/0.803). K3's A
   is not fl((x-2)*R) (0.79 against a conditional base of 0.78). Three separate rationals, one scheme.
6. **Why coordinate search cannot move, in one sentence.** In K1 every constant sits on its own sum's grid
   (`review2/k1_lattice.py`: u(S_k)/ulp(c_k) in {1, 2, 4} for all 14 stages except D0, which is 2^20 finer and is
   exactly -1/16). A stored sum's rounding sequence therefore does not depend on its constant's bits at all; it
   depends on the product's low bits, i.e. on the upstream stage value. A constant enters the output only through
   the next product's phase, n_k*y mod 1 with n_k the integer mantissa of S_k: a hash of the full stage value. So
   the single-coefficient exact-match landscape (`review2/k1_sawtooth.py`) is a 1-ulp-wide V whose depth is the
   stage's visibility (0.05% per ulp for N6, 10% per ulp for N2), and AN2, ss_fc and grid_a are local maxima in
   every coordinate (`review2/k23_midscan.py`; grid_a's p5 is pinned to the ulp, +-1 ulp costs 25 points). There
   is no lattice of near-equivalent points and no plateau bits in K1: the information is in the phases, per run.

### 11.2 Critique, point by point
1. **"Information-theoretic" (wc/README 3, 7) describes the linearised phase solve, not the data.** The solve
   measures each line's phase with a template that assumes the candidate's upstream rounding pattern; a wrong
   upstream re-hashes 25-50% of the next sum (`results_rehash.txt`), so the measured phase jitters by a
   pseudo-random amount per unit coefficient move, exactly as W4 found. An exact-template nudge scan is not subject
   to that: it is the simulation, with the nudge as the only free parameter, and it reads the top phases at +-1/16
   with Z above 10 at every anchor (11.1.2). The verdict should read: *not identifiable by linearised single-phase
   demodulation with dyadic anchors*.
2. **Dyadic anchors are the wrong design for the top stages.** At a = p/2^k the product phase frac(dS_1 * a /
   u(P_2)) = frac(m * a / 2) is 0 for every m divisible by 2^(k+1), so the ladders at 1, 1.25, 1.5, 1.75, 1.875 and
   2 read dS_1 only modulo 2, 8, 4, 8, 16 and 1 grid units ("stage values modulo small integers", playbook 8.6).
   This is why the top stages were "weakly constrained" by nine anchors: the design cannot see them. Non-dyadic
   anchors (any generic double) make the same reading a hash of the full m, and four or five of them pin m
   uniquely by enumeration. The P1 runs are non-dyadic anchors and that is why 11.1.2 could settle the top.
3. **The per-anchor paradox of 9.1.2 has a K1 counterpart.** AN2 is 76% exact at 1.95-2.0, 93% near 1, 70-80% at
   the e7 anchors, and 33-38% on the 1.875 rungs and between anchors. The stage values are right where the ladders
   pinned them and wrong in between, because the fitted coefficients interpolate five dyadic anchors with the wrong
   polynomial in the null space. The e10 ladders at 1.875 were never actually absorbed (fitSA is 39% there): the
   linear step failed for the reason in point 1.
4. **W-C's positive controls (wc/README 2-3) were the right idea applied to the wrong estimator**: they show that
   phase rows plus function rows plus a ridge converge to 1e5-1e6 ulps and that exact match is a needle; both are
   true and neither says the phases are unreadable. The control I ran (`results_fm188.txt`) shows the opposite for
   the exact-template reading: two invisible phases recovered to 1/32 from one run with the downstream exact, and
   still recovered (+41 of 8,192) with a wrong D stage at 1.88. It fails near 1 with a wrong downstream (weights
   100x smaller) and it fails from a cold start with all 28 phases wrong (`posctl_fm.py`, `fm_decode.py`: both the
   match count and a template correlation have no gradient then). So readings are valid only where one stage
   dominates the residual; 11.1.3 shows that is the case on about 15 of the 38 P1/P1b runs.
5. **W2's s1-s3 "rows fail the control" and s4/s5 "contradict the law" are the same fact.** s1-s3 (p9..p6) are the
   upstream of s4/s5. The meter for s4/s5 can only be trusted once s1-s3 are right to well under a grid unit at the
   anchor, and nothing in W-A checks that. Note also that grid_a's q7 wants to move (monotone +0.3 points toward
   -6 ulps in `results_k23_midscan.txt`), a direct hint that the D upstream is off.
6. **Two of my own first ideas are dead and recorded as such.** (a) An error-magnitude fingerprint of the flipped
   stage near 2: the output change of a one-grid flip is blurred by the downstream re-hash (std about equal to the
   mean, `results_k1_fingerprint.txt`); no stage label survives. (b) A lattice of near-equivalent coefficient sets
   with period u(S_k)/ulp(c_k): the period is 1-4 ulps, so there is no lattice (11.1.6).
7. **W5 p5's "per-site meters fail when every stage differs" is the same cascade** (K3 re-hash: one ulp of N4 or
   N5 changes 40-50% of the final sum). Its coherent multi-run sum at the true phases is the right instinct; the
   per-run single-stage scan of 11.1.3 is the cheaper form of it, valid on runs where ss_fc is above about 60%.
8. **Oracle-side hygiene.** The four remaining Stirling misses (x = 15.7, 31.4 and 2^51, 1-2 ulps) are not ties and
   deserve one look; and `g_rows.py` now returns 2.96M rows, so every rate in sections 3 and 10 refers to an older
   sample.

### 11.3 Ranked ideas with tests
**I1. Per-run single-stage phase readings, unwrapped over y0, then top-down integer recovery (K1 first).**
- Reading: for a run at y0 where one stage dominates (gain above about 150 points at 8,192 points), the best nudge
  d_k of product k is frac(dS_{k-1}(y0) * y0 / u(P_k)) to +-1/32 (refine the scan around the maximum).
- Unwrap: dS_{k-1}(y) is a polynomial in y with integer-ulp coefficients (the errors of the constants above stage
  k-1, which 11.1.2 says are zero for N7, N6, N5): for D product 5, dS_4^D(y) = dD3 + dD4*y + dD5*y^2 (+ dD6*y^3 if
  the D top is off; D products 2-3 show gains up to 100 points at 1.92-1.94, so include dD5, dD6 as unknowns).
  Enumerate the integer vector over a box of +-5e3 per coefficient (1e8-1e11 cases, vectorised or coarse-to-fine)
  against the readings at the 10-15 runs; the right vector reproduces every reading to 1/32; the wrong ones do not,
  because frac(m*y0*...) is a hash with 1/32 resolution and 10 runs give 32^10 of discrimination.
- Then fix those coefficients, re-score, and repeat one stage up or down (N product 6 reads dS_5^N = dN4 + dN3*y +
  dN2*y^2 given the top is right; N product 5 reads dS_4^N = dN4 + dN3*y).
- Positive control first (mandatory): synthetic truth = AN2 with dD3 = +437, dD4 = -212 (and separately dN3 = +91,
  dN2 = -340), simulated on the real P1/P1b grid; run the scan, the unwrap and the recovery; report the reading
  precision and the minimum gain at which a reading is trustworthy. Then Excel.
- Expected result: a set that is above 90% on 1.70-1.80 and 1.93-1.97 (where the readings come from), with the
  1.80-1.89 band improving as a consequence, since its superposition is made of the same mid-stage errors.
- Cost: one day of W4; no oracle.

**I2. Non-dyadic anchors with rungs (probe V1), for the readings that need more points.** A top-stage or
mid-stage bump of +40 per run becomes +500 over 12 runs at the same y0 (rungs at a + c*2^-j, j >= 24, all share
the top and mid phases to better than 1e-4 grid units). Use them where 11.1.3 found gains of 50-150 (p120-p126,
p131, p145) and for the D top stages at 1.92-1.94. Also the final verifier of I1: a new anchor the fit has never
seen must come out above 90%.

**I3. K2: redo the control with upstream null-direction moves, then read p6/p7/q6/q7 by the same scans.**
- Control: truth = efC + an integer move along the function-null directions restricted to p9..p6, q8..q5 (max 50
  ulps; check the function change is under 0.02 ulp); run W2's s4/s5 meter: it must show the pseudo-random
  per-anchor phases of 11.1.4. That closes the "contradiction".
- Reading: near 3 the top stages of K2 are visible (x^9 = 2e4 against N of order 1: the re-hash of a 1-ulp p6 move
  reaches 2.4% of the final N and the output weight is large), so the exact-template scan of products 2-4 of N and
  D on the runs at 2.85-3.0 (r704, allX cases) should show bumps; positive control with planted p7/p6/q7/q6 moves
  first; then unwrap over the non-dyadic allX anchors (12 of them) exactly as in I1.
- The late plateau bits stay as W-A read them; efC's early refit was done with the s4/s5 phases contaminated and
  should be redone after the upstream is fixed. Expect the (9,8)/(8,8) question to resolve itself (the tiny p9 is
  invisible to every stored value, `results_rehash.txt` row c[0]: 0.000 everywhere).

**I4. K3: the same per-run scans on top of ss_fc**, on the runs where ss_fc is above 60% (0.99-1.07 and the e2xx
runs), reading N products 6-8 and D products 5-8 (the stages whose one-ulp moves re-hash 40-50% of the final sum).
Keep the six read constants fixed. Positive control with planted N4/N5/D4/D5 moves first. Then unwrap with the
e9xx anchors (non-dyadic ones are needed: probe V3).

**I5. (1.5,2]: unchanged plan for W6 (map R's staircase and lines on q401-q414), plus two things from this review**:
R's chain is a third stored Horner pair, so the per-run scans apply once a first candidate exists; and design the
q4 anchors non-dyadic (1.5 and 2- are dyadic; use the sweep points).

**I6. Still valid, lower value:** ties as parity readers for sub-grid constants (N0 mod 4, N6 mod 2, N5 mod 2 in
K1: the only constant bits a sum's rounding can see); the interval LP as the final polish of the last 1-2 ulps per
coefficient, started from a set above 95%.

**Dropped:** the Fisher design for weak lines (I1 reads the same quantities directly), the generator hypotheses
(nothing in 11.1 depends on how the coefficients were made), the error-magnitude fingerprint, the lattice, and all
cross-kernel composition transfer (11.1.5).

### 11.4 Execution plan (Opus workers)
Common rules: exact simulation per hypothesis (`review2/gsim.c` for K1; port the nudge mechanism to the K2/K3
simulators, it is ten lines); positive control before any reading is trusted; report per-run reading tables
(run, y0, exact rate, stage, phase, gain) in a README under the worker's directory; exact match is the final
verifier, never the search objective; score on the full formula-filtered set.

**W4 (K1).** (1) Port `review2/stage_scan.py` to all dense runs (P1, P1b, y1xx, r7xx, q502, e3, e7, e10, e1001-
e1020) and tabulate the readings; (2) positive controls for the unwrap (I1); (3) recover dD3, dD4 (dD5, dD6) from
the D product 5 readings, then dN2, dN3, dN4 from N products 5-6, then rescan; (4) report the by-tenth exact rates;
(5) if the 1.80-1.89 band is still under 80%, run the 28-nudge per-run fit there starting from the corrected set
(it is no longer a cold start) and read what is left; (6) request probe V1 for whatever stage still needs points.
Success: above 90% on every tenth, misses explained by near-ties.

**W2 (K2).** (1) The upstream null-direction control (I3) and a one-paragraph retraction of "contradicts the law";
(2) a nudge scan of N products 2-4 and D products 2-4 on the 2.85-3.0 runs with positive control; (3) unwrap over
the allX anchors, fix p6-p7/q6-q7, redo the early refit (earlyfit2) and the ladder solve; (4) score. Also: a +-50
ulp scan of q7 (the monotone gradient in `results_k23_midscan.txt`). Success: one set above 90% on every anchor
cluster, or a run where no nudge of any stage gains while the set is under 60% exact (that would be the
structural failure W2 suspected, and it would then be documented with the run).

**W5 (K3).** (1) Per-run scans on top of ss_fc on the runs above 60%; (2) positive control; (3) recovery of
N4/N5/D4/D5 (the 40-50% re-hash rows); (4) request probe V3 if the e9xx anchors do not unwrap. The W5 p5 coherent
multi-run sum stays as the cross-check. Success: above 90% near 1 and above 70% far, then iterate.

**W6 ((1.5,2]).** As in w6/README "Next", with q4 anchors kept non-dyadic, and once a first R candidate exists,
the per-run scans. Also run `review2/xk_reach.py`-style tests on R itself (is R a plain quotient, as K2's S is?).

### 11.5 Probe layouts (case names unused as of this review: `kmyst-gam-v1xx`..`v4xx`)
- **V1 (K1 non-dyadic anchors with rungs):** 8 anchors a = 1.0731, 1.2347, 1.3912, 1.4683, 1.5577, 1.6341, 1.7718,
  1.9253 (W4 may replace any by a run from 11.1.3 whose gain was 50-150). Per anchor 12 runs of 8,192 consecutive
  doubles starting at a + 1.37*2^-j, j = 46, 44, ..., 24; three cases of 32,768 rows per anchor, 24 cases
  `kmyst-gam-v101`..`v124`, column A = y, D = GAMMA(A). About 50 minutes of oracle.
- **V2 (K1 unwrap pairs):** 12 cases `kmyst-gam-v201`..`v212`, 32,768 rows each: two runs of 16,384 consecutive
  doubles per case starting at y0 and y0 + 0.0008 for y0 = 1.7050, 1.7230, 1.7580, 1.7690, 1.7920, 1.8010, 1.9320,
  1.9450, 1.9550, 1.9680, 1.9800, 1.9920 (between AN2's crossings, 9.2.3); D = GAMMA(A). Pairs 0.0008 apart give
  the phase derivative of each reading, which fixes the sign of the unwrap.
- **V3 (K3 non-dyadic anchors):** 6 anchors a = 0.7413, 0.8127, 0.9318, 1.1731, 1.2869, 1.4137, each 12 runs of
  8,192 at a + 1.37*2^-j, j = 46..24 (3 cases per anchor, `kmyst-gam-v301`..`v318`); B = GAMMALN.PRECISE(A),
  C = LN(A).
- **V4 (K2 near-3 rungs for the top stages):** 3 anchors a = 2.8731, 2.9417, 2.9873, 12 runs each as above
  (`kmyst-gam-v401`..`v409`); B = GAMMALN.PRECISE(A), C = LN(A).
Fill V1 and V2 first (they unblock W4); V3/V4 after the W5/W2 positive controls say how many points a reading
needs.

### 11.6 What this review settles
- fl53(fl64) at every op is validated by data (79 points on the Stirling kernel).
- K1: N7, N6, N5 of AN2 are right to within a few ulps; the open error is in the mid stages and is readable per run.
- K2: the s4/s5 anomaly is an upstream re-hash artefact of the single-phase meter, not a law violation.
- K2's inner and K3's inner are plain stored quotients; no composition links the three GAMMALN kernels.
- Dyadic anchors cannot pin the top stages; the next probes are non-dyadic.

## 12. Third review (Fable): what the K2 hand-off signature is, and is not

State at 2026-10-20 03:30 UTC. Scripts and a README are under `review3/`.
- Every number below was computed on committed oracle data with exact x87 simulation (numpy longdouble,
  `astype(float64)` after each stored op).
- grid_a is used exactly as committed in `w2/grid_a.txt`.
- Data: the long consecutive runs in (2,3] (r701, r112, p401, r702, r610, r901, r703, r113, r704, r611, r705, v401a,
  v404a, v407a), 8,192 or 16,384 doubles each.
- I did not run the oracle and pushed nothing.

The brief asked for early-evaluation code shapes that inject 2-3x Horner noise at s4/d3 and leave the rest of the chain
plain. Before brainstorming I measured what such a shape must satisfy, and the measurements exclude the whole category.
- 12.1 gives the facts.
- 12.2 gives the candidate list with verdicts and op sequences, so nobody re-screens them.
- 12.3 gives the surviving explanation and the cheap screens.
- 12.4 is the cross-kernel critique.
- 12.5 says what is settled.

### 12.1 Headline: the stage-4 sum is a plain stored Horner sum, and the "noise" is not pre-rounding noise
1. **Excel has the plain-Horner N.S4 and D.S3 sum lines, at the plain-Horner frequency, phase-coherent, at full
   amplitude.** `review3/freq.py` fits the sum line of s4 = fl(P4 + p4) and d3 = fl(P3 + q3) to 1e-5 cycles/step on each
   long run, in Excel - exact rational and in sim(grid_a) - exact rational. The last column is the frequency the line
   would have if the head were a two-part sum lacking the x^3 slope.

   | run | x0 | N.S4 f_base | Excel f | sim f | Excel/sim power | two-part-head f |
   |---|---|---|---|---|---|---|
   | r701 | 2.0612 | 0.40542 | 0.40542 | 0.40542 | 196x / 230x | 0.40329 |
   | r112 | 2.2361 | 0.41158 | (other line dominates) | | 2897x / 2782x | 0.40907 |
   | p401 | 2.3300 | 0.41490 | 0.41490 | 0.41490 | 256x / 209x | 0.41218 |
   | r610 | 2.4568 | 0.41939 | 0.41940 | 0.41939 | 78x / 78x | 0.41637 |
   | r901 | 2.5000 | 0.42093 | 0.42093 | 0.42092 | 121x / 91x | 0.41780 |
   | r703 | 2.6123 | 0.42493 | 0.42493 | 0.42493 | 88x / 79x | 0.42151 |
   | r113 | 2.7183 | 0.42872 | 0.42880 | 0.42880 | 1630x / 1685x | 0.42501 |
   | r704 | 2.8765 | 0.43439 | 0.43428 | 0.43427 | 64x / 79x | 0.43025 |

   - D.S3 behaves the same: Excel f = sim f = f_base on every run, powers within 30%.
   - The line's frequency is s4'(x)·2^-51/ulp(s4), i.e. the slope of the full cubic head p0 x^3 + p1 x^2 + p2 x + p3
     times x.
     - It is not shifted by the x^3-term slope (0.002-0.004 cycles/step, 200-400 resolution elements).
     - It is not replaced by an x-independent line at 8·p3 = 0.3351, the signature of a (p4 + p3 x) pair. In
       `spec2.py` the power at 0.3351 equals the sim's on every run.
   - The per-run phase offset, Excel minus sim, is -0.30..+0.32 cycles and differs by run. That is the per-anchor
     smooth offset of P4 + p4 that W2's single-anchor fits absorb.
2. **The pre-rounding jitter of P4 + p4 is at most about 0.1 ulp.**
   - A sawtooth with pointwise phase jitter of sigma s cycles keeps a fundamental of exp(-2 pi^2 s^2).
   - The measured Excel/sim amplitude ratios (0.8-1.4, mean 1.07) bound s < 0.1 u.
   - A pre-rounding noise of 0.27 u, which is what "sigma 0.30 against 0.13" would mean, would leave 24% of the line.
   - So the hand-off reader's extra sigma is not noise in P4 and not noise in the sum's operand. It has to be a change in
     WHICH WAY the sum rounds on a subset of points.
3. **Per point, everything is reachable from the stored chain with +-1 or +-2 at the hand-off** (`flips.py`, late
   chain = grid_a from s5/d4).
   - 99.8-99.9% of Excel's outputs on r112/r113 equal fl(t·fl(N/D)) for some |dN|, |dD| <= 2 ulps of s4/d3.
   - 23-27% of points need a shift. Of those, the minimal shift is N +-1 for 68-74%, D +-1 for 17-26% and N +-2 for
     2-6%.
   - The N side carries the anomaly, as the (5,3) versus (4,3) fingerprints already said.
4. **The needed shifts are not a constant per cluster, in either sign or size, and they are phase-locked to the s4
   sawtooth.**
   - `clusters.py`: the best constant integer (dN, dD) per 512-double cluster gains nothing over (0, 0) on r112 and r113
     (0.766 -> 0.766, 0.730 -> 0.730).
   - 7-9% of points need +1 while others in the same cluster need -1. A synthetic smooth head offset gives no sign
     mixing.
   - `dgridlike.py` reproduces W2's fingerprint (an 80-bit head plus a fractional pre-rounding offset in 1/8 ulp per
     cluster). It gives REAL 0.821 / 0.813 / 0.844 / 0.761 on r112 / r113 / r901 / r703, against
     0.912 / 0.900 / 0.939 / 0.932 for an all-stored grid_a truth scored the same way. That reproduces W2's deficit with
     the committed grid_a.
   - The need-shift rate by octile of frac(P4/u) is concentrated in the two quarters around the sum's midpoint, where the
     u/4-quantised P4 produces exact ties and ties-to-even decides:
     - r112: 0.56 1.32 1.25 1.54 1.49 0.67 0.56 0.62;
     - r113: 0.84 0.87 1.52 1.46 1.06 0.96 0.70 0.59.
5. **Unstored-op variants are followed by Excel at a rate that depends on the run** (`keeptest.py`: where a variant of
   the stored chain differs from it, which one does Excel follow). Each cell is "follows variant / follows stored chain";
   the remainder matches neither.

   | variant | r112 (2.236) | r901 (2.500) | r703 (2.612) | r113 (2.718) |
   |---|---|---|---|---|
   | Nmul4 unstored (P4 at 64 bits) | 0.54 / 0.41 | 0.79 / 0.16 | 0.48 / 0.45 | 0.83 / 0.14 |
   | Nadd4 unstored (s4 kept) | 0.55 / 0.39 | 0.54 / 0.40 | 0.75 / 0.19 | 0.86 / 0.12 |
   | Dmul3 unstored | 0.68 / 0.17 | 0.44 / 0.43 | 0.49 / 0.33 | 0.55 / 0.23 |
   | head products N1-4, D1-3 unstored | 0.58 / 0.36 | 0.68 / 0.25 | 0.48 / 0.43 | 0.79 / 0.16 |
   | whole-chain products unstored | 0.06 / 0.76 | 0.07 / 0.78 | 0.06 / 0.75 | 0.11 / 0.71 |

   - A structural difference would give the same rate on every run.
   - A rate that swings from 0.48 to 0.83 with x is what a run-dependent sub-ulp offset of P4 + p4 does to the tie class.
     With offset 0 the stored chain wins everywhere; with offset u/8 the "true side" wins on 75% of the tie points; with
     u/4 on 50%.
   - This is review 2's re-hash mechanism (11.1.4) seen through a different meter, and it is the same thing W2 calls
     "2.3x noise".

### 12.2 Candidate code shapes considered, with verdicts
Conventions:
- p/q are highest degree first; stage i = multiply by x, then add c[i]; `sd()` = store to double.
- head = stages 1..4 (N) and 1..3 (D); tail = stored raw-x Horner.
- The predicted "first-rounding line" is the frequency of the sawtooth of the sum in which c4 (q3) enters, in cycles per
  x-step, at x = 2 / 2.5 / 3.

| shape | op sequence (N head; D analogous) | predicted hand-off sigma N / D | predicted line | verdict |
|---|---|---|---|---|
| M-A bottom-paired Estrin (Estrin cubic/quartic) | `A=sd(sd(p3*x)+p4); r=sd(sd(sd(p0*x)+p1)*x+p2); s4=sd(A+sd(r*x2))` | 0.30 / 0.29 | 0.3351 flat, partner 0.068-0.104 | dead: no 0.3351 line, N.S4 line present (12.1.1) |
| M-B even/odd in x^2 (fdlibm style; W2's closest) | `E=sd(sd(sd(sd(p0*x2)+p2)*x2)+p4); O=sd(sd(p1*x2)+p3); s4=sd(E+sd(O*x))` | 0.33 / 0.17 (D's even part 0.219-0.222 sits below 0.25, half grid) | 0.066-0.099 | dead: D side too smooth (W2's "3 points too smooth"), N.S4 line present |
| M-C x^3 term added last | `X=sd(sd(p3*x)+sd(p2*x2)); A=sd(X+p4); s4=sd(A+sd(p1*x3))` | 0.30 / 0.29, slow partner | f_base - 8·3 p1 x^2 (0.002-0.0045 lower) | dead: Excel's line is at f_base to 1e-5 |
| M-D pair trees, power sums with c4 entering before the last addition | any order with >= 2 sums at the s4 grid | 0.41+ | doublets near f_base | dead: too noisy and the doublets are absent; W2's 120 orders agree |
| M-E root-pair `p3*(x+a)`, a = p4/p3 | `A=sd(sd(x+a)*p3)` | 0.39 / - | exactly 0.25 (fl(x+a) on grid 2^-49) | dead: no period-4 line (spectra), N.S4 line present |
| split constant c4 = a + b, correction terms, c0 x^4 added separately | `s4=sd(sd(P4+a)+b)` | 0 extra (b constant or slow) | none | cannot produce noise; a slow term is a per-run constant |
| x-jitter in the head (x' = x + O(ulp)) | `P4=sd(s3*x')` | 0.3 / 0.14 | none | dead: would reduce the line to 10% (12.1.2) |
| products unstored in the head (x87 `fmul`/`fadd`/`fstp` codegen without /Op) | keep Nmul1-4, Dmul1-3 | tie flips 12.5% (x<2.76) / 25%; D 6% / 12% | none | disfavoured: the 80-bit-head model fits real data worse than it fits a stored truth (12.1.4), and the follow rate is run-dependent (12.1.5) |
| head all 80-bit, stored once (combo m=8) | `s4=sd(head_80)` | as above, plus removal of s3 jitter | N.S3/D.S2 lines absent | disfavoured for the same reasons; W2's refit under it gave 85.5% against a 98.6% control |
| sub-grid late residues (p6 +12 ulps etc.) | | 0 | none | output-identical (`clusters.py`: exact 1.000 against grid_a); only plateau constants move |

Why the earlier screens missed nothing:
- Every member of the "different association" family either removes the s4 sum line, moves it by >= 0.002
  cycles/step, or adds an Excel-only line. None of these is in the data.
- Every member of the "rounding-mask" family makes the hand-off smoother than a stored Horner relative to an 80-bit
  head. The data are rougher.

### 12.3 What does produce exactly this signature
A stored raw-x Horner head whose pre-rounding value P4 + p4 (and P3 + q3) differs from grid_a's by a smooth, sub-ulp,
anchor-dependent offset. Through the u/4 quantisation of P4 and ties-to-even, such an offset:
- flips the stored s4 by +-1 on a phase-selected subset (both signs, the middle quarters of the sawtooth);
- leaves the sum line's frequency and amplitude untouched;
- shifts its phase by a run-dependent constant;
- reads as "2.3x noise" in any reader that assumes grid_a's upstream values.

All five facts in 12.1 follow. So the question is not "which op is different" but "why can no single early coefficient
set produce the offsets at all anchors at once". Two explanations remain; W2 has stated the first itself
(`w2/x87tools/README.md`, caveat).
- **(a) grid_a's late or cross-N/D coefficients are displaced along function-null directions.** A joint integer move of
  several late coefficients, mixing N and D:
  - keeps the function and every plateau residue c_k mod ulp(S_k);
  - is not found by single-coordinate needle scans (review 2 showed every candidate is a local maximum in every
    coordinate);
  - forces the head to compensate differently at each anchor.

  W2's own controls show the symptom: with the late part frozen on a displaced truth, early-only fits stall at 34-67%.
- **(b) The single-anchor 99.97% fits lock onto aliases** *(Correction, 2026-10-20: the "99.97%" figure was W2's per-point s4read admissibility, the same quantity as the 99.8% reach of `flips.py`, not a coefficient fit. W2's best single-window coefficient fits are 82-85%. The conclusions in 12.6-12.8 that do not depend on it stand.)*, so their per-anchor offsets are not Excel's and the joint
  solve inherits wrong rungs. The "2.5 cluster" figure comes from ladder rungs whose P4 phase is the same on every rung,
  so a single anchor cannot test the rounding structure. Only consecutive runs can (r901 at 2.5 gives 78.4%).

Cheap screens, in order:
1. **Per-anchor offset meter from the tie class** (no phases, no unwrap; minutes per run).
   - For each long run, compute the triple of 12.1.5 (follows variant, follows stored, neither) for Nmul4 and Dmul3.
   - For a grid of offsets delta in [-u/2, u/2], in 1/32 steps, add delta to the pre-rounding P4 (P3_D) of the stored
     model and compute the same triple from simulation.
   - The delta that reproduces the real triple is Excel's hand-off offset modulo u/4 at that anchor. `flips.py`'s octile
     profile is the cross-check.
   - Positive control: plant a p3 move and recover it.
   - Output: delta_N(x_i) and delta_D(x_i) on the 14 long runs and the 1,024 dense clusters.
2. **Consistency test on the offsets.**
   - Fit delta_N(x) = x·(dp0 x^3 + dp1 x^2 + dp2 x + dp3) + dp4 with integer ulp moves of the early coefficients:
     5 unknowns, dozens of congruences mod u/4, by enumeration or LLL.
   - If a solution exists, the early coefficients follow and the joint-fit failure was (b).
   - If none exists within the function's box, it is (a): the late chain must move.
3. **For (a): a function-null search over the late and cross-N/D coefficients, scored by `flips.py`'s per-point
   admissible-shift tables** rather than by exact match.
   - The score is the fraction of points needing no hand-off shift at (4,3), summed over anchors.
   - The right displacement collapses the 23-27% need-shift rate toward the base level.
   - W2's `ysearch.py` function-preserving segment search is the engine; the objective is new.
4. **Rerun the fingerprint with the exact late set used for REAL.** Applying "p5 -4 ulps, q4 -1 ulp" literally to the
   committed grid_a drops r112 from 0.766 to 0.172 exact, and the +-2 reach from 0.998 to 0.955 (`flips.py 1`). So either
   those units are not coefficient ulps, or the REAL fingerprint row was computed with a different late set. Worth one
   line of clarification in `w2/x87tools/README.md`.
5. **If 1-3 fail: test the x87 control word.** The only x87 behaviours not covered by stored/unstored masks are the
   rounding mode and the precision control word.
   - A truncating control word at the hand-off would show as a one-sided offset of -0.5 ulp at every anchor in the meter
     of step 1.
   - A 53-bit precision control word differs from stored only on 64-bit near-ties, and is already excluded by 11.1.1.
   - Neither is expected, but step 1 tests both for free.

### 12.4 Cross-kernel critique
- **K1 (GAMMA's G).** It comes from a different code generation era: GAMMA arrived in 2010, and GAMMALN.PRECISE equals
  legacy GAMMALN at 100% (P4). Review 2 showed the top N stages exist as stored doubles with AN2's phases.
  - Nothing in 12.1 transfers except the method. The tie-class meter of 12.3 step 1 works on K1 too: products sit 1-2
    binades below sums, so the tie classes are 1/4 and 1/8 of the points, and it gives per-run stage offsets without
    phase unwrapping.
  - The frozen status and the I1 route (per-run readings, top-down integer recovery) stand. No oracle spend on K1 before
    W4 reports the unwrap control.
- **K3 (near 1).** W5's unstored-op hot spots, N.S5 and D.S3 in W5's ascending numbering, are the sums that add the fifth
  coefficient from the top of N and the fourth of D.
  - These are the same chain positions as K2's Nadd4 and Dadd3: the first sums whose product is quantised two or more
    binades below the sum.
  - By 12.1-12.3 that is where a smooth upstream offset first turns into phase-selected +-1 flips, so these hot spots are
    coefficient symptoms, not structure. W5's "hand-off model not diagnostic" result is consistent.
  - Apply the tie-class meter and the per-point reach test (`flips.py` with ss_fc at (4,3)) before any further
    structural screen.
- **(1.5,2].** The W6 pipeline's positive controls are on stored-Horner synthetics. Since K2's head is a stored Horner
  after all, there is no structural reason to doubt them.
  - The 27-32% candidate is at the stage where exact match is a needle. The stall is the lattice problem W6 documented,
    not a hidden op.
  - One cheap check from this review: W6's 16 crisp tracks all have |dF/dx| >= 0.015. An exactly flat track would mean a
    pair sum (c + c' x) and would contradict the stored-Horner reading, so its absence is a mild confirmation.
- **Common cause.** The three GAMMALN rationals share one failure mode. It is methodological, not a shared codegen
  quirk.
  - Every pipeline freezes the late coefficients on plateau residues and the function, then searches the head.
  - But the function-null space contains integer cross-N/D moves of late coefficients that preserve both residues and
    function, and change the roundings.
  - Controls validate the search under that assumption, so they cannot see it. W2's own displaced-truth controls
    (34-67% ceilings) are the exception and show the symptom.
  - Lifting the freeze (12.3 step 3) is the common fix. K1's problem is different in kind: mid-stage phases are readable
    per run but not yet unwrapped.

### 12.5 What this review settles
- Excel's K2 head is evaluated as a stored raw-x Horner through stage 4 (N) and 3 (D): the sum lines are at the Horner
  frequency to 1e-5 cycles/step with full amplitude. Every different-association shape (bottom-paired Estrin, even/odd,
  x^3-last, pair trees, root pairs) is excluded by data, in all variants.
- The "2.3x / 3x noise" is not pre-rounding noise (jitter bound 0.1 ulp). It is phase-selected +-1 flips from a
  run-dependent sub-ulp offset of the hand-off sum, acting on the quantised product and ties-to-even. The stored-chain
  reader and the 80-bit-head fingerprint both convert that offset into apparent noise.
- Unstored-op structures at the head are disfavoured: Excel is rougher than a stored Horner relative to an 80-bit head,
  and the follow rates swing with x.
- The open problem is coefficient consistency across anchors, with late or cross-N/D displacement as the leading cause.
  The tie-class offset meter and the per-point admissible-shift objective are the cheap tools for it; both need only
  committed data.
- One hygiene item: the committed grid_a with "p5 -4, q4 -1" applied as coefficient ulps is far worse than grid_a. The
  REAL fingerprint's late set should be stated exactly.

### 12.6 Follow-up: the tie-class offset meter, run (2026-10-20 05:30 UTC)

The coordinator asked for 12.3 step 1 as `review3/meter.py`, with its positive control, on the 14 long runs. Done, plus
the screens the result called for.
- Scripts: `review3/meter.py`, `meter_keep.py`, `latescan.py`, `ties.py` (README table).
- Everything below uses committed data and grid_a as committed; no oracle, nothing pushed.

**Meter.** One smooth offset per chain, in ulps of the stored sum of a chosen stage, injected either:
- before the product's quantisation (`pre`: what an upstream coefficient difference looks like); or
- on the constant (`post`: a sub-grid residue error of that constant).

It scans [-3.5, 3.5] in 1/32 steps by coordinate ascent, with exact match as the objective.

Controls: planted p3 +3, p4 +1, q2 -16 ulps read at (4,3) `pre`; p5 +1, q4 +2 read at (5,4) `post`.
- Every planted offset is recovered to 1/32 ulp, with the synthetic truth going from 3-15% to 97.7-100% exact.
- Expected dN = 3x/8 + 1 and dD = -x. Read: +1.844/-2.250 at 2.236, +1.938/-2.500 at 2.5, +2.062/-2.875 at 2.877.
- The reader is sound.

**Real runs, (4,3) pre.** Columns: base -> best exact; dN and dD in ulps of s4 and d3; "flat" is where the rate stays
within 0.5 points of the best.

| run | x0 | base -> best | dN [flat] | dD [flat] |
|---|---|---|---|---|
| r701 | 2.0612 | 0.703 -> 0.715 | +0.12 [-0.06,+0.19] | -0.12 [-0.16,-0.06] |
| r112 | 2.2361 | 0.766 -> 0.769 | +0.00 [0.00,+0.06] | -0.06 [-0.16,+0.09] |
| p401 | 2.3300 | 0.761 -> 0.763 | -0.12 [-0.12,+0.19] | -0.06 [-0.22,+0.06] |
| r702 | 2.3457 | 0.712 -> 0.749 | +0.59 [+0.41,+0.62] | +0.25 [+0.19,+0.38] |
| r610 | 2.4568 | 0.775 -> 0.807 | +0.25 [+0.19,+0.28] | -0.22 [-0.34,-0.16] |
| r901 | 2.5000 | 0.784 -> 0.799 | +0.06 [-0.03,+0.22] | -0.12 [-0.28,-0.09] |
| r901 | 2.2500 | 0.731 -> 0.755 | -0.03 [-0.03,-0.03] | +0.31 [+0.16,+0.38] |
| r703 | 2.6123 | 0.737 -> 0.750 | -0.03 [-0.06,+0.03] | +0.19 [+0.09,+0.25] |
| r113 | 2.7183 | 0.730 -> 0.752 | -0.25 [-0.34,-0.19] | +0.00 [-0.06,+0.09] |
| r704 | 2.8765 | 0.759 -> 0.775 | +0.03 [0.00,+0.06] | -0.12 [-0.12,-0.09] |
| r611 | 2.9012 | 0.782 -> 0.810 | -0.06 [-0.19,-0.03] | +0.16 [+0.12,+0.19] |
| r705 | 2.9568 | 0.757 -> 0.781 | +0.06 [+0.06,+0.09] | -0.22 [-0.25,-0.03] |
| v401a | 2.8731 | 0.737 -> 0.772 | -0.34 [-0.44,-0.34] | +0.16 [+0.06,+0.19] |
| v404a | 2.9417 | 0.664 -> 0.694 | +0.31 [0.00,+0.38] | -0.38 [-0.41,-0.31] |
| v407a | 2.9873 | 0.675 -> 0.701 | +0.59 [+0.50,+0.62] | -0.12 [-0.22,0.00] |

Reading:
- The offsets are small, their signs wander with x (dN/x from -0.12 to +0.25), and they buy 1-4 points, where a true
  hand-off offset is recovered to 98%.
- **So the per-anchor mismatch is not a smooth sub-ulp offset of the hand-off sums.** My 12.3 "smooth offset" wording
  was too strong. The per-anchor phase offsets of 12.1.1 are real, but they are not what separates Excel from grid_a.
- Section 12.3 step 2 (fit the offsets to a quartic) is moot.

**Screens that followed, all negative:**
- **(5,4) and (6,5) `post`:** zero gain on every run. grid_a's p5, p6, q4 and q5 sub-grid residues are right.
- **`latescan.py`:** p6-p9 and q5-q8 moved by whole units of their sums' grids (m in [-6, 6]), scored with the (4,3)
  meter on top.
  - m = 0 is the per-run optimum on all 11 runs, for every one of them.
  - p5 and q4 are degenerate with the head offsets (moves of +-2 cost nothing) and show no consistent preference.
  - So there is no late integer-grid displacement, and explanation (a) of 12.3 fails in its single-coefficient form.
- **Unstored-op variants from stage 5 on** (Nadd5, Nmul5+Nadd5, Nmul6, Nadd6, Dadd4, Dmul4+Dadd4, Dmul5, N or D
  unstored into the division): Excel follows the stored chain on 60-80% of the differing points, and the variant on
  1-23%.
- **Division and final-op variants** (N*fl(1/D), single-rounded N/D, fl(fl(t*N)/D), S unstored): Excel follows the
  stored form; the variant gets 1-7%.
  - So the chain from stage 5 on, the division and the final product are as modelled.
- **Keep-mask heads with the meter on top** (`meter_keep.py`: P4 and/or P3_D unstored, or all head products unstored,
  plus (4,3) offsets): best rates 0.71-0.83. Not a structure:
  - better than stored on some runs (r112 0.77 -> 0.83, r113 -> 0.80, v407a -> 0.79);
  - worse on others (r901 at 2.25: 0.76 -> 0.73, r611 0.81 -> 0.78, r705 0.78 -> 0.75).
- **Controls for the meter itself:**
  - A random +-1 flip of s4 on 25% of points gives the meter nothing, and so do unstored-op truths.
  - A jittered head offset (p2 -40) is recovered to 93-97%.
  - The 5-11 point gains the (5,4) `pre` setting shows on real data also appear on head-offset truths (0.55 -> 0.86), so
    they carry no structural information.

**Where the deviation lives** (`ties.py`, exact 64-bit positions). Each point is classified by the exact position of
the stored-chain sum P4_q + p4 on the u grid:
- ties at 1/2, quarters at 1/4 and 3/4, exact at 0;
- 25% each for N, where P4's grid is u/4; for D, positions step by 1/8.

The question is which neighbour of grid_a's s4 reproduces Excel's output where the two differ downstream. These
"decided" points are about half of each class.

| run | class | decided | Excel agrees with round-to-nearest/even | with the true side of the unquantised product |
|---|---|---|---|---|
| r112 | tie 1/2 | 2091 | 0.449 | 0.518 |
| r112 | 1/4 | 2105 | 0.609 | 0.609 |
| r112 | 3/4 | 2077 | 0.842 | 0.842 |
| r112 | 0 (no rounding) | 2103 | 0.981 | 0.981 |
| r901 | tie / 1/4 / 3/4 / 0 | 711 / 782 / 703 / 739 | 0.353 / 0.648 / 0.925 / 0.771 | 0.668 / same / same / same |
| r113 | tie / 0 | 3048 / 3198 | 0.299 / 0.852 | 0.659 / 0.852 |
| r703 | tie / 1/4 / 3/4 / 0 | 744 / 726 / 734 / 747 | 0.489 / 0.627 / 0.634 / 0.657 | 0.500 / same / same / same |
| r704 | tie / 0 | 1464 / 1377 | 0.530 / 0.802 | 0.523 / 0.802 |

Three facts follow.
1. On the exact ties, Excel agrees with ties-to-even only 30-53% of the time, and with the true side of the 64-bit
   product 50-67%. Neither rule holds, so on these points Excel's stored s4 is not fl(P4_q + p4) with grid_a's P4_q.
2. On the quarter classes, where round-to-nearest is unambiguous, Excel disagrees on 8-39% of the decided points.
3. On the exact class (position 0, no rounding at all), Excel's s4 is still one ulp off on 2-34% of the decided points.

Facts 2 and 3 cannot be a rounding-rule difference at the sum.
- They mean **Excel's pre-rounding sum P4 + p4 sits in a different position class than grid_a's on 10-35% of the points,
  point by point**: Excel's P4_q differs from grid_a's by one or two quarter-steps in a non-smooth pattern.
- P4 = fl(s3*x), with s3 on the u/8 grid. So a point-wise +-1 ulp(s3) difference in s3 does exactly this: it moves P4 by
  +-x/8 u (one or two quarter-steps) on the subset of points where Excel's P3 + p3 rounds the other way. Such a
  difference is a different rounding pattern of the stage-3 sum, coming from different early coefficients.
- That is upstream jitter from the early coefficients, not a hand-off structure. It is why every hand-off reader (W2's
  s4read sigma, the dgrid fingerprint, my meter) sees "noise". It is also why W2's single-anchor fits, which move p0-p3
  and q0-q2, can reach 98.6-99.97% while no per-anchor offset can.

**Consequences.**
- *(Correction, 2026-10-20: the "99.97%" figure was W2's per-point s4read admissibility, the same quantity as the 99.8% reach of `flips.py`, not a coefficient fit. W2's best single-window coefficient fits are 82-85%. The conclusions in 12.6-12.8 that do not depend on it stand.)*
- **12.3's two explanations are narrowed.** It is neither a smooth offset at the hand-off (this section) nor a late
  displacement (latescan, residue scans, unstored variants).
  - It is the early coefficients p0-p3 / q0-q2 themselves, whose errors reach the hand-off as point-wise jitter through
    the stage-2/3 roundings.
  - The information that pins them is in the rounding patterns of P2, s2, P3 and s3 (grids u/128 to u/8), which the
    output sees only through this jitter.
  - That is W2's original problem statement, now with the structural alternatives removed.
- **The K2 structure is settled** as far as these data can settle it: stored raw-x Horner, both chains, every op, with
  fl(t*fl(N/D)) outside. No further structural screen is warranted on K2.
- **The (1.5,2] and K3 pipelines should not spend time on hand-off structures either.** The same reader applies there if
  anyone wants the per-anchor offsets, with the same caveat: small offsets mean "early coefficients", not "structure".
- **A practical route for the early coefficients, if the task continues:**
  - Per-anchor exact fits over (p0-p3, q0-q2) on consecutive runs. Not ladder rungs, whose phases repeat.
  - Each fit gives the integer vector modulo the per-anchor aliases; then a lattice over the anchors.
  - `ties.py`'s class tables are a sharper per-anchor objective than exact match, because they separate the jitter
    (classes 1/4, 3/4 and 0 disagreeing) from the tie resolution.
- **Hygiene:** `meter.py --control` prints the expected readings. The D expectation uses ulp(q2) = 2^-58, which is what
  the meter read. My first draft used 2^-59 and looked like a factor-2 error; it was not.

**Coordinator note (2026-10-20).** W2's own offset-meter controls (`w2/x87tools/README.md`) recovered
function-preserving EARLY-only displacements (p0..p4, q0..q3; scales 10-100 ulps) to 0.92-0.96 on every run, while real
data reach only 0.71-0.83. W2 therefore leans to a joint late/cross-N/D displacement. This section leans to early
coefficients, after single-coefficient late scans came out null. The two readings must be reconciled: an early-only
synthetic truth must reproduce real's per-class `ties.py` disagreement rates, and real's 0.71-0.83 meter ceiling, before
either explanation is accepted.

### 12.7 Early-only versus joint null displacements: neither reproduces the real class pattern (reviewer 3)

The question: W2 observed that function-preserving early-only displacements of p0-p4/q0-q3 at 10-100 ulps are recovered
by the offset meter to 0.92-0.96 on every run, while real data stall at 0.71-0.83. Does that argue for a joint
late/cross-N/D displacement instead of "early coefficients"?

The test (`review3/ties_truth.py`):
- It builds function-null integer moves from the SVD of the relative-function Jacobian of (x-2)N/D in coefficient ulps
  (600 points on [2.01,3]).
- It runs the position-class table of 12.6 on synthetic stored-Horner truths displaced along those moves, scored against
  grid_a, next to the real data.
- Each entry is the agreement with round-to-nearest-even per class (tie, 1/4, 3/4, exact), on decided points.

| truth | moves (ulps) | exact vs grid_a (r112/r901/r113/r703/r704) | N classes tie, 1/4, 3/4, 0 (r112; r901) | D classes (r112; r901) |
|---|---|---|---|---|
| REAL | | 0.77/0.78/0.73/0.74/0.76 | 0.45 0.61 0.84 0.98; 0.35 0.65 0.92 0.77 | 0.23 0.84 0.86 0.86; 0.48 0.77 0.62 0.78 |
| early-only, 2 softest, scale 30 | p0+17 p1+28 p2+2 q0+30 q1+6 q2+1 | 0.90/0.90/0.88/0.89/0.87 | 0.81 0.81 0.97 0.98; 0.78 0.74 0.96 0.95 | 0.54 0.62 0.96 1.00; 0.51 0.57 0.96 1.00 |
| early-only, 2 softest, scale 100 | p0+58 p1+92 p2+6 q0+100 q1+20 q2+2 | 0.74/0.75/0.69/0.72/0.67 | 0.51 0.44 0.92 0.89; 0.53 0.32 0.92 0.74 | 0.54 0.10 0.89 0.54; 0.50 0.10 0.93 0.22 |
| joint, 4 softest, scale 30, seed 1 | p0-11 p1-3 p6-30 p7-2 p8+7 p9+23 q0-3 q1-1 q6-3 q7-13 q8-26 | 0.64/0.27/0.60/0.29/0.70 | 0.96 0.94 0.97 0.94; 0.36 1.00 0.14 1.00 | 0.85 0.82 1.00 0.78; 0.59 0.47 0.98 0.38 |
| joint, 4 softest, scale 30, seed 3 | p0-30 p1-23 p2-3 p6-2 p7+1 p8-1 p9-12 q0-23 q1-7 q2-1 q7+5 q8+15 | 0.87/0.89/0.85/0.87/0.84 | 0.75 0.97 0.74 0.97; 0.69 0.97 0.69 0.97 | 0.45 0.96 0.71 0.97; 0.48 0.95 0.52 0.97 |
| joint, scale 100 | (p6-100, q8-86, ...) | 0.27-0.46 | near chance everywhere | near chance everywhere |

Two families are omitted: early-only directions that exclude the near-invisible p0, and the third-softest early
direction. Neither is function-preserving at 30 ulps (0.27-0.48 exact, chance everywhere).

Reading:
1. **Early-only null displacements give a one-sided pattern.** At the scale that matches the real exact rate
   (100 ulps):
   - one quarter class is far off (N 1/4 0.30-0.44, D 1/4 0.10-0.12) while the other is near-perfect (3/4 0.89-0.93);
   - the D exact class is far off (0.13-0.54).

   That is the signature of a smooth pre-rounding offset at the hand-off, which is exactly what the offset meters
   absorb. W2's 0.92-0.96 recovery of such truths is expected and says nothing about the real data.
2. **Joint late/cross-N/D null displacements are erratic by run:** the same move scores 0.27 to 0.70 on different runs.
   They give either near-perfect hand-off classes with the misses elsewhere (seed 1 at r112: N 0.94-0.97, D 0.78-1.00,
   at 64% exact), or again a one-sided pattern (seed 3: 1/4 at 0.95-0.98, 3/4 at 0.38-0.74). Not the real pattern.
3. **The real pattern is two-sided and symmetric:**
   - both quarter classes are off by similar amounts;
   - the exact classes are mostly right (N 0.66-0.98, D 0.78-0.93);
   - the tie class is at or below chance. At r901/r113, N is at 0.30-0.35 while the true side of the unquantised product
     scores 0.66-0.67; on the other runs both rules are at chance.

   No smooth displacement of any coefficients, early or late, produces that. Point-wise +-1 flips of the stage-4/3 sums,
   in both directions, on 10-30% of the points do, and on such a pattern every offset meter is blind (12.6, random-flip
   control).
4. **A two-sided point-wise pattern cannot come from coefficient differences inside grid_a's own head.**
   - Every head constant sits on its sum's grid, so coefficient moves act smoothly at their own stage. They only turn
     into flips downstream through the product roundings, with grid ratios that make them one-sided and small.
   - The only two-sided source, the stage-1 product rounding set by p0's value, reaches s3 at the 0.3% level.
   - So the honest answer is "neither": W2's control result does not establish a late/cross displacement, and 12.6's
     "early coefficients" wording was also too strong.
   - What the position-class data show is a two-sided, zero-mean, point-wise re-decision of the hand-off sums. Its only
     correlate found so far is the parity-linked tie behaviour on r901/r113. That is consistent with an unquantised
     stage-4 product there, which `meter_keep.py` shows is not the whole story (0.80-0.83 at best).
5. **Caveat on the method:** a mismatch of +-1 ulp(y) caused elsewhere can be "decided" by an s4 neighbour by
   coincidence. The joint seed-1 truth calibrates this: 36% misses from late stages leave the N classes at 0.94-0.97, so
   the contamination is a few points, far below the real 8-39%.

### 12.8 W2's truths, the mixture test, and determinism (coordinator summary, 2026-10-20 03:05 UTC)
- **Class tables on W2's own synthetic truths.** Truths: cE30 and cE100 (early-only function-null moves); cL30 and
  cL100 (joint late + cross-N/D). Each is scored against grid_a on r112, r113, r901 and r703 (reviewer 3).
  - All four are ONE-SIDED: one quarter class is near-perfect and the other far off (cE30: N 1/4 0.94 vs 3/4 0.29-0.45;
    D 1/4 0.88-0.92 vs 3/4 0.07-0.10).
  - cE30 matches real's exact rate on every run, yet its pattern is opposite to real's TWO-SIDED pattern: N
    |1/4 - 3/4| = 0.00-0.27 and D 0.02-0.15 for real, against 0.49-0.85 for cE30.
  - So the offset meter recovering early-only truths is expected, and neither displacement family is real's mechanism.
- **Determinism.** Coordinator check: 2,054,772 distinct x in (2,3], of which 551,309 are evaluated more than once in
  different cases and rows. Zero conflicting outputs.
  - W2: a row-index analysis of the q2 data (row mod 2..1024, row blocks, case, part) is at binomial noise. The excess
    variance is run-to-run in x.
  - Reviewer 3: v401a/v404a/v407a hold the same x (ladder rungs 1e-13 apart) in rows 1-8192 and 8193-16384. The two
    halves agree to three decimals.
  - Excel's K2 output is a deterministic function of x. No thread, chunk or row effect.
- **Mixture-of-paths test (reviewer 3).**
  - Pure single rounding (SSE2 / PC=53) differs on 0.1-0.3% of points, and Excel follows double rounding there.
  - Fused-everywhere and all-80-bit are rejected (follow rates 0.05-0.14).
  - The only alternative Excel follows above chance is the stage-4 product entering the sum unquantised (identical to a
    fused s3*x+p4). Its follow rate is flat within each run and swings between runs: 0.83 at 2.718, 0.79 at 2.5, 0.54 at
    2.236, 0.48 at 2.612. That is the signature of a fixed structure seen through a per-anchor sub-ulp offset, not a
    per-point switch.
  - No low-bit, parity, row or autocorrelation predictor exists.
- **Standing recommendation.** K2 = stored x87 Horner, with at most the stage-4 (N) / stage-3 (D) product unquantised.
  The remaining 17-20% of points are decided by the sub-ulp positions of P4+p4 and P3+q3, i.e. by p0-p3 / q0-q2.
  Recovering them needs an exact per-anchor fit over p0-p3 / q0-q2 on consecutive runs, with the stage-4/3 product's
  quantisation as a free bit, followed by a lattice intersection of the per-anchor integer vectors (modulo aliases)
  across anchors.

### 12.9 The reach test on W2's truths: the late chain is right to well under 1% (reviewer 3, 2026-10-20 03:16 UTC)
The test: the fraction of outputs the stored grid_a chain reproduces with a hand-off shift of at most 1 or 2 ulps at
(4,3), and the fraction of misses that NO such shift reaches.

| source | exact vs grid_a (r112/r901/r113/r703) | reach, at most 1 ulp | reach, at most 2 ulps | unreachable misses |
|---|---|---|---|---|
| REAL | 0.766/0.784/0.730/0.737 | 0.995/0.988/0.983/0.995 | 0.998/0.999/0.999/0.998 | 0.7%/0.7%/0.4%/0.9% |
| cE30 (early-only) | 0.75/0.78/0.72/0.74 | 1.000/1.000/0.992/1.000 | 1.000 everywhere | 0% |
| cE100 | 0.40/0.45/0.37/0.40 | 0.91/0.86/0.78/0.83 | 1.000/1.000/0.9998/1.000 | 0% |
| cL30 (joint late+cross) | 0.14/0.46/0.82/0.56 | 0.61/0.79/0.997/0.88 | 0.903/0.955/1.000/0.978 | 11%/8%/0%/5% |
| cL100 | 0.03/0.18/0.43/0.16 | 0.17/0.41/0.79/0.39 | 0.48/0.69/0.93/0.68 | 54%/38%/12%/38% |

Reading:
- A late displacement leaves misses that no hand-off shift reaches: 5-11% of misses at scale 30 and 38-54% at scale 100
  (except where the move happens to be weak, as cL30 at r113). Head-level deviations leave none.
- Real leaves 0.4-0.9%. So any late component is at least an order of magnitude smaller than cL30's, and the late chain
  is right to within effects on well under 1% of points.
- The K2 deviation is a head-level, point-wise, two-sided re-decision, just not a smooth one.
- A joint head+late solver must constrain its late moves accordingly: they may change fewer than about 1 in 100 outputs
  beyond hand-off reach.

Reviewer 3's mechanism analysis:
- On x87 nothing lies between a 53-bit store and the 64-bit register for a product: the precision control word offers
  only 24, 53 or 64 bits.
- Split multiplies (coordinator screen m1-m5), chop rounding, an 80-bit s3 operand, and the product error reaching P5
  through an unrounded s4 (killed by the s4 sum line) all fail to give "partial" following.
- So the leading candidate is a point-wise re-decision ONE stage earlier, at s3. A +-1 ulp(s3) change moves P4 by about
  a quarter-step and changes the s4 position class exactly as observed.

The direct read is a table of Excel's shift on the s4 exact class, against e3 = P3_80 - P3_q in 1/16 steps jointly with
the eighth-position class of P3_q + p3, per run. The positive control is a planted p2 move.

### 12.10 The s3 and D-side tables are flat; what K2's deviation must be (reviewer 3, 2026-10-20 03:28 UTC) — SUPERSEDED by 12.13 (re-hashed head line withdrawn by reviewer 3)
Runs: r112, r901, r113, r703, all against grid_a. Script: `review3/s3table.py`; the D-side table is a variant that was
not committed.

**s3 table** (the stored chain's s4 exact class).
- **The deviations are 2-3 ulp(s3), not 1.**
  - Excel needs s4 +-1 on 9-17% of points.
  - The minimal s3 shift that explains each such point is mostly +-2 or +-3 ulp(s3): for example, 4.7% / 3.3% of exact
    points on r112 and 11.4% / 2.2% on r703.
- **Flat across all covariates.** Rates are flat within noise across the 32 cells of P3 eighth-position x parity x
  sign(e3), and across e3 in 1/16 bins. Among s3 +-1 shifts, sign(k) agrees with sign(e3) only at 0.44-0.59, i.e. chance.
- **Controls.** p2 +5 / +40 ulps give one-sided s4 +1 that is flat across classes: smooth upstream offsets are
  one-sided and class-flat, whereas real is two-sided and class-flat.

**D-side table.**
- Against N's exact class, D's hand-off geometry has no predictive power.
- D re-decides its own ties (position 4/8) at 13-34%, the same rate N does. This is not a cross-chain rule.
- The mirrored table (D's exact class against N's geometry) is flat.

**Deterministic x-jitter rules.** Round trips in the head ((x+k)-k, (x*m)/m, (x/m)*m, exp(log x), sqrt(x)^2) are all
rejected: where they differ, Excel follows the stored chain 0.52-0.78 of the time and the rule only 0.06-0.42.

**Synthesis (12.6-12.10).**
- Excel differs from grid_a at the stage-4/3 hand-off sums by +-1 u on 15-30% of points:
  - two-sided, zero-mean, and deterministic in x;
  - including points where the stored sum is exactly representable.
- None of these predicts it:
  - x bits or row position;
  - the geometry of any earlier stage (P1-P4, s1-s3, d1-d3);
  - the cross chain;
  - any smooth per-run offset.
- The needed pre-rounding change is >= u/2, which excludes every x87 rounding, product-resolution and operand-precision
  variant.
- The late chain is right to within 1% of outputs (12.9).
- These are the properties of a DIFFERENT polynomial value at s3/s4, of order 2-3 ulp(s3), point-wise and not a function
  of grid_a's own intermediate geometry. One family produces exactly that:
  - head coefficients differing from grid_a's by enough (hundreds to thousands of ulps on p0-p3, allowed by the function
    null space) that their s2/s3 rounding patterns are unrelated to grid_a's;
  - late coefficients compensating below the reach threshold.
- Additive offsets (hoffF) cannot emulate this, because they add constants rather than re-hashing the chain.
- The tool is W2's joint solver, with the 12.9 reach bound as its constraint.

### 12.11 Coordinator note (03:45): the 12.10 synthesis is falsified, and K2 and (1.5,2] converge

- **W2 re-hash calibration (K2).** The planted truths were grid_a with large function-null head moves:
  - p1 moved by +8.9e3, +8.9e4, +8.9e5 and +8.9e6 ulps;
  - the remaining head coefficients were LS-compensated on [2,3];
  - late stages were left at grid_a.
- **Fits recover the plants.** Per-run additive head-sum offsets (hoff.py, 8,192 points per run) recover every plant:

  | truth | r113 start → fit | r901 start → fit |
  |---|---|---|
  | rh1e5 | 0.793 → 0.968 | 0.798 → 0.982 |
  | rh1e6 | 0.910 → 0.924 | 0.901 → 0.958 |
  | rh1e7 | 0.899 → 0.950 | 0.741 → 0.979 |
  | REAL | 0.742 → 0.812 | 0.784 → 0.842 |

  - A re-hashed head is therefore offset-recoverable, so "re-hashed head + reach-bounded late" does not explain real.
- **Joint v5.** This adds head offsets, s5/d4 fractional offsets and full-residue late moves. Results:
  - Real reaches r113 0.861, r703 0.907, r112 0.921 and r901 0.953.
  - The control (cLH) reaches 0.999.
  - The fractional offsets settle at 0.
  - The gains come from integer p5/q4 moves of ±1..4 that disagree in sign across runs. Under the 12.9 reach bound these moves compensate for head error.
- **Conclusion for K2.** Every planted family is recovered to at least 0.92-1.00. This includes head re-hashes up to 1e7, late or cross moves within reach, and stage-4/3 fusion. Real is capped at 0.81-0.95 and needs extra freedom that is inconsistent between runs. So stored raw-x Horner with any tested coefficient change, with or without stage-4/3 fusion, is NOT the structure.
- **W6 at (1.5,2].** The same picture appears there:
  - Excel's N-chain line frequencies match to 1e-10.
  - Every phase-using (stage-VALUE) lattice pass disagrees with the others by 1e6..1e9 u(S5).
  - The ladder plus function rows leave at least 8 weak directions in N5..N9, each 0.9e5..4.6e5 u.
  - The control closes at 99.97% (W6 README item 28). The Excel blocker is item 29.
- **Joint reading.** In both kernels the stored-Horner VALUE model fails at the middle stages, while the stage FREQUENCIES (the multiplication structure) are right.
- **Next tests.**
  - W2 runs a window-length scan: fit per window of W points, from W = 64 to 8k, against controls at the same W.
    - If the fit reaches 1.0 as W shrinks, a hidden quantity varies slowly in x (piecewise tables or breakpoints).
    - If the fit stays capped, the freedom is per point (a different op or precision).
  - W6 runs an absolute-phase smooth-shift test on the K=0 and L runs. It also checks whether phases are constant within a window but drift between anchors, and whether the mismatch is a fixed fraction of a cycle (a different precision at one stage).

### 12.12 W2 locality and precise-head screens (03:50-03:55): both negative

- **Window-length scan (winscan.py).** Joint fit per window of W consecutive doubles.

  | W | REAL | controls (cH, rh1e6, cLH) |
  |---|---|---|
  | 64 | 0.885 | 0.95-0.98 |
  | 256 | 0.859 | 0.95-0.98 |
  | 1k / 4k | 0.851 | 0.95-0.98 |

  - The extra freedom is **point-local**: no piecewise tables, breakpoints or slowly varying hidden quantity.
- **Low-bit screen (bitfeat.py, exact rational).** The features were exact, tie, guard, near-tie and quarter positions at 53 bits, exact-at-64, the double-rounding window and the fl24 half.
  - Nothing beyond propagation sensitivity and the known D tie effect.
  - Hidden extended precision on a single op, or float32 anywhere, is excluded.
- **Precise heads (vheads.py).**
  - Variants:
    - V1N: exactly-rounded N head;
    - V1D: exactly-rounded D head;
    - V1ND: both.
  - CompHorner and double-double hi are identical to V1ND on every point.
  - Results:
    - Excel follows the variant on 0.49-0.88 of the points where it differs; the planted control is followed at 1.00.
    - That lifts the exact rate by only 1-8 points.
    - An exact block through s5/d4 and power sums (forward and reverse) are worse than stored.
  - Rejected.
- **Net.** Excel's K2 head behaves like stored Horner, re-decided point-wise partly toward the exact value. The strength of that re-decision varies by run.
- **Queued next.**
  - W2: the follow-strength map vs x, and head forms H1-H4 (shifted variable, hi+lo coefficients, Dekker-split x, two-order combination).
  - Reviewer 3: asked for a concrete structural form.

### 12.13 Reviewer 3: proof-grade ceiling for grid_a's op structure (supersedes the 12.10 "re-hashed head" line)

- **Fact 1.** A run spans about 2e-12 relative in x. Within it, any coefficient move acts on every stored op as a constant pre-rounding offset, however large the move. So the "re-hashing" decorrelation does not exist.
- **Fact 2.** On (2,3], ulp(p_k) = ulp(s_k) for k = 1..4.
  - An integer coefficient move therefore shifts s_k by exactly that integer, with no re-decision.
  - Only the products P2, P3 and P4 can re-decide.
  - The point-dependent span those re-decisions can create at v4 is at most 0.6 u (0.85 u above x ≈ 2.76).
  - Excel's two-sided ±1 exact-class re-decisions need a span of at least 1 u.
- **Exhaustive sweep (`review3/sweep.py`).** Per run it scans c2 and c3 on a grid, finds c4 exactly by interval coverage, then finds (g2, g3) for D, and verifies the optimum by direct simulation.
  - Supremum of {grid_a op structure, any coefficients}:

    | run | x | grid_a → c4 only → (c2,c3,c4) → +(g2,g3) |
    |---|---|---|
    | r112 | 2.236 | 0.766 → 0.812 → 0.814 → 0.829 |
    | r901 | 2.500 | 0.784 → 0.811 → 0.826 → 0.848 |
    | r113 | 2.718 | 0.730 → 0.775 → 0.796 → 0.809 |
    | r703 | 2.612 | 0.737 → 0.746 → 0.753 → 0.767 |

  - The optimal hand-off offsets disagree in sign across runs, so not even one polynomial lies behind the per-run optima.
- **Lattice (`review3/lattice.py`).** The head exact-null lattice has rank 3-4 and is trivially enumerable. By Fact 1 every element is a per-run constant, already covered by the sweep.
- **Consequence.** The missing 15-23% per run is **op structure**, not coefficients. All K2 coefficient and offset solvers are stopped.
- **Next.** Any candidate structure is first screened on whether its head has a point-dependent span of at least 1 u at the hand-off. Candidates queued:
  - H1: shifted-variable head;
  - H2: hi+lo coefficients;
  - H3: Dekker-split x;
  - H4: two-order combination;
  - H5: a variable or scaling under which the head sums re-decide.
- **Model-free read.** The admissible-s4 map per point is requested from reviewer 3.
- **Per the user's directive** (bit-exact, no DEVIATION acceptance), grid_a is not shipped.

### 12.14 After the 12.13 ceiling (04:00-04:06): every structure caps the same; Excel's head noise is about 2× ours

- **K2 supremum sweeps (W2 ssweep/shoff, matched to sweep.py within 0.006).** All of these cap at the raw-x values 0.829/0.842/0.813/0.766 (r112/r901/r113/r703), ±0.003:
  - N head as z = x−c (c = 1, 2, 2.5, 3);
  - N head as Dekker-split x;
  - the same variants on the D head, independently;
  - per-run offsets at the division and the final multiply.
  - A planted z-head is recovered 1.000 by its own sweep and only 0.96 by the raw-x sweep, so the method separates structures.
- **Output δ-profiles are flat** at y and at R = n/d. Misses are ±1..±6 ulp(y), and only 37-49% are ±1. So the discrepancy is not at the final ops.
- **Hand-off read (hread3.py).** Unique-m4 points (12-18%, selection-biased). |s4_Excel − h*| quantiles:
  - median 0.46-0.58 u (stored 0.25);
  - 90% at 1.1-2.1 u (stored about 0.5).
  - It is uncorrelated with our stored error and with all geometric covariates, and has a run-dependent mean bias.
  - So Excel's effective head noise is about 2× ours and independent of it.
- **W2 bound.** The late chain pins E4/E3 to under u/8 on [2,3]. Extra higher-degree terms are therefore at most 1e-9 in raw x and cannot add head noise. A common factor would change the late coefficients.
  - So the higher-degree and cancellation class is excluded within raw-x Horner.
- **W6 parallel, (1.5,2].**
  - The per-chain phase fits are control-level.
  - But with those late stages fixed, no head coefficients make N/D equal Excel's R (5e-15 rms vs 2.3e-16).
  - Augmented outer-form LS is running (k·N/D, +c, +c(x−2), shifted (x−1), Cody-like, extra leading terms).
- **Next.**
  - W2 screens power-table heads (repeated multiply, squaring, exp/ln), heads at a round-tripped x' = fl(fl(x−a)+a), and N/D heads built from shared subexpressions.
  - Reviewer 3 is asked for structural forms that escape all of the above.
