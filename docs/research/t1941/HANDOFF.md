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
