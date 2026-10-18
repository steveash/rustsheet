# T-1778: reverse-engineering Excel's small-x erf kernel (research plan and state)

Living document for reviewers. Goal: **bit-exact** `ERF.PRECISE(x)` for 0 <= x < 0.5, then the rest of the
family on the same kernel (ERFC for x >= 0.5, NORM.S.DIST, NORM.S.INV, BETA.DIST). Reviewers: add alternative
hypotheses and experiments in the section at the end ("Reviewer suggestions"); the research agent reads it back.

Where things live:
- Research branch `agent/research/T-1778-kernel-map`: probe cases, scripts and the canonical write-up.
- Probes: `eval/suites/probes-numeric/cases/kmyst-*.case.json` with `.oracle.json` sidecars.
- Write-up: `docs/excel-semantics/statistics.md`, section "One incomplete-gamma kernel ... (T-1778)".
- Scripts: `docs/research/t1778/` (README there); `docs/research/t1778_incomplete_gamma_kernel_map.py`.

## 1. Data

All values come from our own oracle, `rustsheet-oracle fill -b excel-graph`. That is current Microsoft 365
Excel for the web, a 64-bit build, with en-US settings. None of the data comes from older workbooks.

| Probe family | What it samples |
|---|---|
| `kmyst-norm-erfc-a/b`, `kmyst-erf-gamma`, `kmyst-kernel-map` | Cross-function identities (see section 2.1). |
| `kmyst-erf-tiny`, `kmyst-erf-mantissa`, `kmyst-erf-consecutive`, `kmyst-erf-scale` | Tiny x (below about 1E-8). |
| `kmyst-erf-stride06..41`, `kmyst-erf-runs`, `kmyst-erf-zstride`, `kmyst-erf-zsweep*`, `kmyst-erf-bridge`, `kmyst-erf-ladder`, `kmyst-erf-zladder`, `kmyst-erf-scatter` | Runs of consecutive x mantissas at chosen z = x². |
| `kmyst-erf-wsweep`, `kmyst-erf-wsweep2` | 21 "fine sweeps". Each is 24 runs of 128 consecutive mantissas, stepping z by 6E-17. They cover z from 5E-12 to 4E-3. |
| `kmyst-erf-wladder1/2` (new) | 40 more fine sweeps, geometric in z (ratio 1.16, z = 1.2E-5 to 3.9E-3). |
| `kmyst-erf-wbig`, `kmyst-erf-xcross`, `kmyst-erf-xcross2`, `kmyst-erf-wlong`, `kmyst-erf-wlong2`, `kmyst-erf-wknee1..4` | Long runs (1k to 32k points) for local rates and the grid crossing near x = 0.148. |
| `kmyst-erf-twoarg` | Two-argument `ERF(a,b)`. |

How much information each point carries: one Excel result pins the hidden "lo" quantity (section 2.3) only to
about one rounding tooth. Each point is therefore close to a single bit. Everything is fitted over runs of
thousands of points by maximising "cover": the number of points whose result is consistent with the model
under one free offset.

## 2. What is established (high confidence)

### 2.1 One kernel

- `ERF(x) = GAMMA.DIST(x^2, 0.5, 1, TRUE) = CHISQ.DIST(2x^2, 1, TRUE)`, bit-exact.
- `NORM.S.DIST(x) = ERFC.PRECISE(-x/sqrt2)/2`, bit-exact.
- erf is odd. `ERFC(-d) = fl(1 + ERF(d))` for d < 0.5.

So the kernel works in z = fl(x*x). It is the incomplete gamma P(0.5, z) path.

### 2.2 Tiny x (below about 1E-8)

`ERF = fl( round53(round64(x*A)) + x*B )`, evaluated through 80-bit (x87-style) double rounding. It matches
3,043 of 3,048 points; the 5 misses are final-sum near-ties.
- A = 1.1170334248975802 (0x3FF1DF5E70C7956B), a double.
- B = 0.0113457421979325066854 (64-bit precision).
- A + B = 2/sqrt(pi) + d0, with d0 about 1.07E-16.
- The 80-bit evidence: the three hard misses of the plain double form all have x*A within 0.0002 ulp of a
  tie, which is inside the 2^-12 window where an 80-bit round makes a tie.

### 2.3 Small x, z up to about 4E-3

The model:

    erf(x) = fl80( fl53(fl64(x * hi)) + x * lo )

- **hi = fl(H(z))** is a stored double, and its step positions were bootstrapped from z = 2E-15 to 4.3E-3:
  - H = A + h1 z + h2 z^2 + h3 z^3 + h4 z^4.
  - h1 = 0.14068131264(22..36), h2 = 0.04481917(05..09), h3 = 0.0018050(2..5), h4 between 9E-5 and 2.7E-4.
  - H rises with z, while erf/x falls.
  - The new sweep fits show H drifting from this polynomial by 0.1 to about 2.6 ulp for z between 8E-4 and
    4E-3, so h3 and h4 need refitting.
- **lo = K*(1 - fl(V(z))) + R(z)**:
  - fl(V) is a second stored double with V near 0.99.
  - Its rounding makes a sawtooth in lo with jumps of K*2^-53, where K is about 1.13669732977.
  - V(0) = V0 = 1 - B/K = 0.990018677883022.
  - V'(0) = c1 = (h1 + 2/(3 sqrt pi))/K = 0.4546572672773 (+-3E-14). This tie means lo's whole linear trend is
    carried by V.
  - V''(0)/2 = c2 = -0.0598953, while the value tied to erf's own z² term is -0.0598389. The difference is a
    smooth part of lo, R of about 6.4E-5 z², that bypasses V.
- **Smooth sum:** H + K(1 - V) + R = erf(x)/x + d0, to better than 1E-16 everywhere probed.
- **Grid crossing:** V rises with z and crosses 1.0 exactly where lo = 0 (x about 0.14838, z about 0.022017).
  Above that point the sawtooth jumps are twice as large and come at half the rate, which fits fl(V) moving
  from the 2^-53 grid to the 2^-52 grid. So lo behaves like K*(1 - fl(1 - t)): the fine part is rounded to an
  **absolute** grid, the `fl(1 + d) - 1` pattern.
- **K and n:** at z = 0, B = round64(K * n * 2^-53) with n an integer. The best candidate is
  K = 1.1366973297689353, n = 89903757133374. Neighbouring lattice solutions (±2.1E-13 in K) are also
  possible.
- **Rate check:** with the tied c1 and K, and R = 0, the V phases at twelve sweeps from 2E-15 to 3E-7 are
  predicted with no free parameter.

### 2.4 Other facts

- `ERF(a,b)` is the difference of the two rounded single values, so it hides no extra precision.
- Worksheet `-` snaps near-cancelling differences to 0. Its rule is not derived.
- ERFC tail (x >= 4): removing the rounding of a naive exp(-fl(x*x)) cuts the error SD from 115 ulp to
  1.85 ulp. A smooth residual drift of about ±2.5 ulp remains, and its source is unknown.
- Generic shape a: GAMMA.DIST's prefix is exp(a*fl(ln z)) times a factor built from Excel's own GAMMALN. The
  density is fl(CDF * fl(a/z)) at tiny z.

## 3. Negative results (ruled out as exact matches)

- **Textbook algorithms:** Taylor series; CALERF and Cephes rationals in four operation orders; NR and Cephes
  incomplete-gamma series with seven prefixes and six tolerances; perturbed constants within 3 ulp. Best
  result: 41% exact.
- **One-parameter forms of W/V with W(0) = 1:** exp(-bz), 1/(1+bz), the (1,1) Padé, a line,
  2exp(-bz/2) - 1, sech(sqrt(2bz)), 1/sqrt(1+2bz), sqrt(1-2bz). All fail by z = 3E-8. An earlier
  "sqrt survives" claim was an artefact of too few sweeps.
- **Literature:** web searches for A = 1.1170334248975802 found nothing.
- **Mathematical identification:** neither A, K, V0 nor B is a closed form under mpmath `identify`.
- **The V′ "knee" between z = 3E-4 and 6E-4 was a measurement artefact.**
  - The long runs span only about 14 V teeth, so local V′ is pinned to about 7E-4 relative at best.
  - The run-to-run scatter (±2E-4) is at that noise level.
  - The earlier "V′ = c1 + 0.1855 z linear law" from short local-rate runs is similarly weak.
  - Absolute phases are the only robust observable for V.

## 4. Current approach and its blocker

Bit parity needs:
- H to under half an ulp absolutely, for all z < 0.25;
- V's phase (V mod 2^-53, or 2^-52 above the crossing) to about ±0.05 everywhere;
- K;
- then R follows from the smooth-sum identity.

What we do:
- Each fine sweep gives V's phase mod 1, with intervals 0.002 to 0.015 wide when sharp.
- The phases are chained up a z ladder by branch-and-bound over integer tooth counts, with an LP on the
  polynomial coefficients (`wfit.py` / `chain3.py`).
- The new sweep-level joint fit (`sweepfit.c`) fits one hi alias, one fractional hi offset, one V phase and one
  common lo offset per sweep. It gives sharp phases at 25 of the 40 ladder sweeps.

The blocker:
- With the dense ladder, **no smooth polynomial V (degree 4 to 6) fits all sharp phases between z = 1.6E-5
  and 4E-5**, even at ±0.03 tolerance, whichever single sweep is dropped.
- Pairs of sweeps only 3–16% apart in z disagree with the best polynomial by 0.27 to 0.45 teeth.
- Either the older lo-offset ("d-mode") phases are biased, which is likely (it picks hi's integer alias per
  run, and a wrong alias shifts lo by 1.76 teeth), or **V is not a smooth function of z** at the 0.01 to 0.5
  ulp level.
- The next step is to redo the chain using only the sweep-level joint-fit phases.

## 5. Open hypotheses (what V, K, H really are)

1. **Absolute-grid rounding of a double:** lo = K*(1 - fl(1 - t)) with t = fl(T(z)) itself a stored double
   (t about 0.0099 - 0.4547 z, so ulp(t) = 2^-59).
   - This is double rounding: t on a 2^-59 grid, then V on 2^-53.
   - It would quantise V's phase to multiples of 1/64, adding ±0.008 phase jitter that a smooth-V chain
     cannot fit.
   - This could explain the chain contradictions.
   - Test: the phase versus z inside one long run should be a staircase with steps of 1/64.
2. **Naive expm1:** lo = K*(1 - exp(-w(z))), with fl(exp(-w)) stored. This is the loss-of-precision
   pattern, natural in incomplete-gamma code (P = 1 - Q).
   - w(0) = -ln V0 = 0.0100313.
   - A linear w predicts c2 = +0.104, but c2 measured from phases is -0.0599, so w would need curvature.
3. **Two spilled x87 intermediates:** hi and V are x87 register spills, stored to doubles, of one polynomial
   or rational evaluation (MSVC spilling), and lo is formed in 80-bit from fl(V).
   - Puzzle: the oracle is 64-bit Excel, where MSVC long double is a plain double, yet the tie evidence
     says 64-bit-mantissa intermediates exist.
   - Could the 80-bit behaviour come from software extended or double-double arithmetic instead?
4. **Rational form erf/x = N/fl(D):** jump = (erf/x)*ulp(D)/D. This needs D's mantissa = 1.98536 and the grid
   crossing at D = 2^k, which in turn needs a large curvature. It is disfavoured because the crossing sits at
   lo = 0.
5. **Boost-style z*(Y + P/Q):** Boost's 53-bit erf has Q1 = 0.455004, close to c1 = 0.4546573 but not equal.
   Possibly a related minimax family with its own coefficients. Not tested.
6. **K and A are derived constants:**
   - K/(2/sqrt pi) = 1.0073718.
   - A/(2/sqrt pi) = 0.989945.
   - Neither is recognised. Perhaps both come from the split of a series coefficient or from GAMMALN(1.5)
     arithmetic.

## 6. Planned experiments

1. Re-chain V using only `sweepfit` sweep-level phases, with old and new sweeps fitted the same way. If it is
   still inconsistent, test hypothesis 1 directly. Run 32k-point runs at z about 2E-5 and check whether the
   in-run phase is a 1/64 staircase.
2. Refit H for z from 4E-4 to 0.25 from the per-sweep fractional hi offsets, which `sweepfit` now reports, and
   extend the hi step bootstrap past 4.3E-3.
3. Pin K exactly using the sawtooth amplitude at large z (free SAWAMP) and the doubled-grid region above
   z = 0.022.
4. Map the region from z = 4E-3 to 0.25 (x up to 0.5) with sweep ladders.
5. Then: ERFC for x >= 0.5 (the exp(-fl(x²)) rounding is known, the residual drift is not), NORM.S.INV, and
   BETA.DIST.

## 7. Constraints for anyone running experiments

- **Oracle use:**
  - Run `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` only, never with
    `--force`.
  - Never print, log or commit `RUSTSHEET_ORACLE_*` values or token caches.
- **Evidence:**
  - Excel is the spec; no disassembling Excel binaries.
  - Summarise, never copy, AGPL or Polyform code.
- **Repository:** never hand-edit `eval/baseline/` or `quality/*.json`.

## Reviewer suggestions

(Reviewers: add alternative hypotheses, critiques of the method and proposed experiments here.)

### Review 1 (2026-10-17)

**Verdict.** The decomposition in section 2.3 holds up to an independent check. On `kmyst-erf-wknee1` run 0
(x0 = 0.01806640625, 32,768 consecutive mantissas, z = 3.264E-4), with hi = fl(H(z)) at the nominal alias and
the sawtooth's linear rise removed, the feasible lo offset is flat inside every 512-point block to 0.006 to
0.007 teeth, drops by 1.00 (+-0.01) teeth sixteen times (one drop per about 2,000 points; c1 predicts 1,950),
never rises, and shows two hi steps (`docs/research/t1778_staircase.py`). So inside a run there is exactly one
rounded quantity near 1, with one constant multiplier K. The blocker is not the phases; it is what the chain
assumes about V between sweeps.

**1. Method critiques**
1. The ladder cannot pin V's absolute phase across decades of z without the right functional form. Between
   neighbouring wladder sweeps (ratio 1.16) V advances by about 0.16·c1·z/2^-53 teeth: 8E9 teeth at z = 1.2E-5.
   Predicting the next sweep's phase to +-0.3 teeth therefore needs V to 4E-11 relative. Branch-and-bound over
   tooth counts works only because the LP imposes a polynomial, and the LP fails exactly when V has a
   non-analytic feature (a term-count change, a branch, a table node) or when another assumed layer (K constant,
   d0 constant, H's alias and fraction, R smooth) is slightly off. Treat the break at z = 1.6E-5..4E-5 as the
   finding: bisect it with new sweeps and measure the size and sign of the jump in de-trended lo across it,
   rather than re-chaining.
2. A rational V would not break the chain: its Taylor series converges geometrically for z << 1, so a degree-6
   polynomial follows it to far below 1E-11 at z = 4E-5. The break points to something non-analytic, or to a
   wrong K, d0 or tooth size, not to "V is a rational function".
3. Every layer is fitted as a point estimate and then assumed by the next. Report feasible intervals instead
   (A + B, K, V0, c1, each under each rounding variant). Example: A + B lies 3.3E-18 from
   fl(2/sqrt(pi))·(1 + 2^-53). Whether that is inside the feasible interval decides whether the +d0 bias
   belongs to the constant or to the rounding sequence.
4. The textbook sweep (section 3) was scored in plain doubles (`erf_global_fit.c` MODE 0; MODE 1 only rounds
   the final sum through 80 bits). numbers.md ("Division rounds twice", "x87 precision is per operator")
   establishes the operator model: every stored result is rounded to 64 bits and then to a double, and register
   intermediates stay at 64 bits. Re-score each candidate under an explicit store-pattern enumeration (which
   intermediates are stored), not under one rounding mode. Candidates that failed in doubles can pass under
   per-store rounding, and the fingerprint in point 5 filters them before any full scoring.
5. Use the measured fingerprint as the first filter: A = 1.1170334248975802 (a double), B, K = 1.13669733 (not
   2/sqrt(pi)), V0 = 0.990018677883022 (not 1), V'(0) = 0.4546573, two stored doubles with constant
   multipliers, teeth K·2^-53, crossing at lo = 0. It rules out, without emulation: every form whose rounded
   quantity near 1 is multiplied by 2/sqrt(pi) (K != c), which is plain Taylor in Horner form, c·x·exp(-z)·S(z)
   (Marsaglia-like) and the NR and AS 239 series; every "1 - fl(exp(-z))" or Lentz "delta - 1" structure
   (V0 != 1); fdlibm and SLATEC "x + x·y" and Boost "z·1.125 + z·c" (hi(0) is neither 1 nor 1.125); and every
   single-rounding structure (a rational N/D, Clenshaw's 0.5·(b0 - b2)). Write each new candidate's predicted
   (hi(0), lo(0), K, V0) first; emulate only survivors.
6. Numerology done, so nobody repeats it: A + B against 30 constructions of 2/sqrt(pi) (double and 80-bit sqrt
   of a double pi, 1/GAMMA(1.5), exp(-GAMMALN(1.5)), 15- to 17-digit literals) matches none within the fit
   precision except fl(c)·(1 + 2^-53) at 3.3E-18, which 30 trials make about as likely by chance as not. PSLQ
   over {A, B, K, c, V0, c1, h1, 1} with coefficients up to 50 finds only A + B = c. K/c = 1.0073717797,
   A/c = 0.9899450978, V0, c1, h1 and h2/h1 are not closed forms in pi, sqrt(pi), e, sqrt(2) and ln 2.
   B/(K - c) = 1.3639721 is not 3·c1 = 1.3639718. Weak hint only: V0's shortest decimal representation has
   15 digits (an 11% accident), as a 15-digit literal in source would.

**2. Hypotheses to add**
- **H-A, data-dependent term count.** A series or fraction summed until a tolerance test stops it changes its
  term count at fixed z thresholds. The dropped term (up to eps relative, one to two teeth for eps = 2^-53 or
  2^-52) ramps like z^n below each threshold and vanishes above it: non-analytic at exactly the level the chain
  cannot absorb. The computed thresholds land where the fits break. Taylor series z^n/(n!(2n+1)): the n = 3
  term crosses 2^-53 at z = 1.67E-5 (2^-52: 2.1E-5), n = 4 at 3.9E-4 (4.7E-4), n = 5 at 2.7E-3 (3.1E-3).
  The exp(-z)·sum((2z)^n/(2n+1)!!) form: 1.13E-5 (1.43E-5), 2.85E-4 (3.4E-4), 2.05E-3 (2.35E-3). The chain
  breaks at 1.6E-5..4E-5 and H drifts from its polynomial at 8E-4..4E-3. An 80-bit sum with a 2^-64 stop
  would drop terms of 0.0005 teeth, invisible; the visible version needs the sum, or the stop test, in doubles.
  MSVC x87 code generation also spills differently inside a loop, so a loop-based series has its own store
  pattern.
- **H-B, two rounded quantities near 1** (lo = K·(V1 - V2)), each with its own rate. Signature: two interleaved
  sawtooths inside one long run (jumps in both directions, or two spacings). Checked on wknee1 run 0: absent at
  z = 3.3E-4. Keep it only as a check near the crossing and above z = 4E-3.
- **H-C, GAMMA.DIST at shape 0.5 calls the erf code, not the reverse.** Section 2.4 already shows it: the
  density at shape 0.5 takes the generic exp(a·ln z) prefix while the CDF is accurate, and shape 1 is special
  too. So the kernel is erf-specific, and "the kernel works in z = fl(x·x)" (section 2.1) is an inference, not
  an observation: with sqrt(fl(x·x)) = |x| in binary floating point the identities hold in either direction.
  The natural variable may be x (an odd polynomial in x, a table in x, a Taylor expansion about nodes). Probe
  P1 decides it.
- **H-D, the 80-bit puzzle is already resolved** by numbers.md: Excel's own operators round twice, so per-store
  x87 behaviour is a property of the calculation engine whatever the build. Drop the software-extended and
  double-double alternatives of hypothesis 3 unless probe P2 contradicts.
- **H-E, the grid crossing is a candidate interval boundary.** lo = 0 and V = 1 at x* = 0.14838
  (z* = 0.022017). Treat x* as a possible table node or branch point: look for a structural change across z*
  (a new polynomial for hi, a different K, a new sawtooth), not only for continuity.
- **H-F, ERFC tail drift = truncation with a stepping term count.** A sign that alternates over intervals that
  grow with x is what a fixed-tolerance truncated expansion gives: the first omitted term alternates in sign
  with n, and n steps down as x grows. A minimax error curve has no jumps; accumulated rounding is not smooth.
  For the asymptotic series |t_n| = (2n-1)!!/(2x^2)^n the thresholds at eps = 1E-15 are x = 8.4, 9.0, 9.8,
  11.0, 12.6, 15.2, 19.4, 27.2 (eps = 1E-14: 8.8, 9.8, 11.1, 13.1, 16.4, 22.4). The measured sign changes near
  10.0, 12.4, 15.8, 19.6 and 22.9 have the right growth but match no single eps, so the form or the stop rule
  differs (a continued fraction's convergents alternate too). Probe P4 looks for the jumps directly; the earlier
  correlation test used 0.25-wide bin means, which smooth jumps away.
- **H-G, sources not yet emulated exactly** (predict the fingerprint first): Marsaglia 2004 (J. Stat. Software,
  "Evaluating the normal distribution": a table of long-double constants, Taylor steps about integer nodes,
  `while (s != t)` loops); SLATEC DERF/DERFC (Chebyshev in 2z - 1, DCSEVL); NAG S15AEF/S15ADF (Schonfelder);
  AS 66 and AS 239; Hart 1968 tables at 16 to 20 digits; IMSL ERF. Also fetch the retired KB texts for
  NORMSDIST (827369) and GAMMADIST (828133) from web.archive.org: they describe the Excel 2003 methods in
  words.

**3. Probes (cheap, decisive)**
- **P1, x-kernel or z-kernel** (about 1,500 cells): `GAMMA.DIST(z,0.5,1,TRUE)` next to
  `ERF.PRECISE(SQRT(z))` on three runs of 256 consecutive doubles z (near 1E-6, 3E-4 and 0.02), with
  `CHISQ.DIST(2*z,1,TRUE)` as control. Consecutive z map to roots 0.35 to 0.71 ulp apart, so a third to a half
  of consecutive pairs share fl(sqrt(z)). Plateaus in GAMMA.DIST, equal to the ERF.PRECISE column, mean the
  kernel takes a stored x = fl(sqrt(z)) (rounded twice, as SQRT is) and works in x; distinct values at every z
  mean an unstored 64-bit root or a z-only kernel. Either way this feeds the kernel any z directly, without the
  x·x step.
- **P2, register width and the final sum** (about 256 cells): `ERF.PRECISE(x)` at
  x = (1 + m·2^-52)·2^(-1022 + j), j = 0..3, 64 mantissas each. There x·B is subnormal in double arithmetic
  (its rounding is 0.5, 0.25, 0.125 and 0.0625 ulp of the result for j = 0..3) but exact in x87's extended
  range, and the result stays normal (Excel flushes subnormal results). Score against both models; the x87 one
  should win at a visible fraction of the j = 0 and j = 1 points. A different structure there would also show
  a tiny-x branch.
- **P3, term-count thresholds** (H-A): one 32k run, or one 24x128 sweep, straddling each computed z_th (the
  thresholds are exact given the form and eps, so +-2E-15 in z suffices); report the jump in de-trended lo.
  No jump at any threshold retires the series family for good.
- **P4, ERFC jump scan** (H-F): 2,000 consecutive doubles around x = 9.9-10.3, 12.2-12.6, 15.6-16.0,
  19.4-19.8 and 22.7-23.1, after removing the exp(-fl(x·x)) rounding. A jump of about eps relative within a few
  ulps of x is the signature, and its position pins (form, eps).
- **P5, crossing structure** (H-E): sweeps straddling z* on both sides, scored for continuity of H's
  coefficients and of K.
- **P6, exact squares**: inside existing sweeps prefer x with at most 26 significant bits, where fl(x·x) is
  exact, to remove the z-rounding term when comparing x-forms with z-forms.

**4. Process**
- Make the de-trended staircase (position, size and sign of every jump in a long run) the standard observable.
  It is alias-free, needs no chaining, and any candidate must reproduce its jump catalogue. Script:
  `docs/research/t1778_staircase.py` (run from a checkout that has the sidecars).
- Keep a fingerprint table in this document (A, B, K, V0, c1, h1, h2, z*, tooth sizes, feasible intervals) and
  require every candidate's predicted fingerprint before emulation.
- Define the exit criterion now, as decision 0056 did for T-1524: if P1 to P4 show no non-analytic feature and
  the fingerprint eliminates every known source, ship the descriptive model (per-interval H, V and K tables
  with the per-store x87 sequence) as the engine kernel at the parity it reaches, record the residual as a
  known gap, and stop extending the ladder.

### Research update 2 (2026-10-18): the kernel is a rational with two stored doubles

Six parallel investigators ran on the data in section 1. Two of them independently reached the same structure,
and the research agent re-verified the tiny-x part (3,048/3,048 with A and D0 alone):

    ERF.PRECISE(x) = fl53( fl80( co ) / W ),   co = fl53(fl64(x * N)),   N = fl(P(z)),   W = fl(Q(z)),   z = fl(x*x)

- P and Q are polynomials in z; N and W are stored doubles. The final divide is x87 (80-bit, then
  rounded); a plain double divide misses a handful of points.
- P(0) = A = 0x1.1df5e70c7956bp+0 and Q(0) = W0 = 0x1.fada15778ece6p-1 = 0.98994509777494666.
  A/W0 = 2/sqrt(pi) + 1.067E-16, which accounts for the old "d0".
- Fits:
  - tiny x: 3,048/3,048 with no fitted constant (the old A+B model got 3,043);
  - wknee long runs: 253,952/253,952, with N and W each stepping one ulp at a time;
  - every run with zero misses in zs3 (36,864 points), ws (64,512), wla (122,880) and wl2 (z up to 7.4E-3).
- Fitted coefficients:
  - P = A + 0.1406813126423 z + 0.04481917070 z^2 + 0.0018054 z^3 + ... (P is the old H).
  - Q = W0 + 0.45465726643 z + 0.0922778683 z^2 + 0.0104635 z^3 + ~7E-4 z^4.
  - Q's terms were predicted, before fitting, from P and the erf series (Q = P/(erf(x)/x)).
- What this explains: everything in section 2.3 beyond hi is an artefact of linearising x*N/W.
  - K, V0, c1, c2 and R come from that linearisation.
  - The true jump size N/W^2 varies with z, so no smooth V could chain; this is the section 4 blocker.
  - The grid doubling at "lo = 0" is W crossing 1.0.
  - P rises because Q rises faster than erf/x falls.
- Ruled out:
  - the phase quantisation of hypothesis 1 (tooth edges are linear in z to 1-6E-4 teeth);
  - CALERF (q1/q0 = 0.45095 against 0.459275 measured);
  - Boost's z*(Y + P/Q).
- Normalised ratios (q1/q0 = 0.459275, q2/q0 = 0.093215, p1/p0 = 0.125942, p2/p0 = 0.040123) sit close
  to the [4/5] Padé approximant of erf(x)/x in z (0.459171, 0.093132, 0.125837, 0.040075). So it is
  probably a degree-(4,5)-like minimax rational. Why Q(0) is scaled to 0.98994 rather than 1 is open.
- Separately, the absolute hi alias is now pinned to about 0.02 ulp up to z = 0.0111. The old H drifted
  10.6 ulp at z = 4E-3, not the 0.1-2.6 stated in section 4.

Next steps:
1. Pin P and Q exactly as doubles from absolute step positions over z up to 0.25 (x < 0.5); this needs new
   full-mantissa ladders above z = 0.011.
2. Determine the Horner order and precision of each polynomial.
3. Identify the source: Remez (4,5), Hart tables, and the reason for Q(0)'s scaling.
4. Test the same "two stored doubles, then an 80-bit divide" pattern on ERFC (x >= 0.5) and on its tail drift.

Review 1's H-A (a term-count change in a series) is not needed for x < 0.5 under this model, but it stays
open for the ERFC tail drift (H-F).

### Research update 3 (2026-10-18): one global formula at 99.90% for x < 0.5; ERFC above 0.5 is open

Small x (round 2):
- **New data.** New full-mantissa ladders (`kmyst-erf-rladder1..7`, 152,576 cells, z from 0.004 to 0.25) fit the
  model with 0 misses per run (146,432 points). Both the 80-bit product and the 80-bit divide are required.
- **No structural change at W = 1.** W = fl(Q) passes 1.0 with only the ulp change; Q(z) = 1 at
  x = 0x1.2fe204020425fp-3 (±15 ulp).
- **One global formula.** P is a quartic and Q a quintic. They are evaluated by x87 Horner with every operation
  stored to double (mode 0300 in `docs/research/t1778/round2/sc.c`).
  - With the coefficients in `round2/best_coef.txt` this reproduces 770,405 of 771,176 points (99.90%) with no
    per-run freedom.
  - Every miss is 1 ulp, and about 90% of them sit within 0.05 ulp of an N or W rounding midpoint.
  - Only A and W0 are pinned exactly. The data alone cannot pin p2..p4 and q2..q5 to single doubles.
- **Source not identified.** No source construction reproduces the coefficients: minimax on any [0,b] under
  four error criteria, Chebyshev interpolation, shifted Padé, or other degrees.
  - Once q1 is matched, all of them miss q2 by about 8E-7 normalised.
  - Excel's rational is 13 to 180 times less accurate than the best possible one, yet still accurate to 1E-21.
    That fits a Remez run below double precision whose coefficients drifted.
  - Hart's tables could not be reached through the proxy.

x >= 0.5:
- The switch to a separate erfc kernel e lies in (0.49922, 0.5171), consistent with 0.5. There,
  ERF(d) = fl(1 - e) at 541 of 541 points and ERFC(-d) = fl(2 - e).
- e looks like fl(exp(-fl(x*x))) times a rational in x with every Horner step rounded to double, judged from its
  noise fingerprint. Cephes' coefficients do not reproduce the ±3 ulp drift.
- The drift's sign changes suggest an approximation interval of about [8, 26.64].
- Five proposed probes (66,240 cells) are specified in the round-2 erfc-structure report: decouple, the exact 0.5
  split, a branch scan, Excel's EXP alongside ERFC, and a dense drift map.
