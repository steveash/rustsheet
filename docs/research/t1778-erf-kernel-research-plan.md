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
- The new sweep-level joint fit (`swfit.c`) fits one hi alias, one fractional hi offset, one V phase and one
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

1. Re-chain V using only `swfit` sweep-level phases, with old and new sweeps fitted the same way. If it is
   still inconsistent, test hypothesis 1 directly. Run 32k-point runs at z about 2E-5 and check whether the
   in-run phase is a 1/64 staircase.
2. Refit H for z from 4E-4 to 0.25 from the per-sweep fractional hi offsets, which `swfit` now reports, and
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
