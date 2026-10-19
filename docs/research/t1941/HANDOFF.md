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
