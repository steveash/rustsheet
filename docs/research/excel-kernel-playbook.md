# Cracking Excel's numeric kernels: a playbook

This playbook is for agents who need a numeric function (ERF, GAMMALN, BETA.DIST, NORM.S.INV, EXP and the
like) to match Microsoft 365 Excel to the last bit. It distils what T-1778 learned while making ERF, ERFC and the
kernels they feed bit-exact on about 10 million oracle points: the patterns Excel's numeric code follows, the probe
designs and analysis methods that worked, the traps that wasted days, and a checklist. Several kernels are still
open (see "Open kernels" at the end). They were very likely written by the same Microsoft team in the same style,
so start from these priors.

The detailed record is in `docs/research/t1778-erf-kernel-research-plan.md` (updates 2-10) and in the T-1778
entries of `docs/excel-semantics/statistics.md` and `numbers.md`. Scripts, coefficient files and verifiers are under
`docs/research/t1778/` on the research branch `agent/research/T-1778-kernel-map`; the engine task brings the
coefficient files to main.

## 1. What Excel's numeric kernels look like (priors)

Every solved kernel followed these patterns. Assume them until the data says otherwise.

1. **x87 extended precision, stored to double after every operation.** Each multiply and add is rounded first to
   a 64-bit mantissa (an x87 register), then to a 53-bit double when stored: fl53(fl64(.)). This double rounding
   is observable. Single-rounded products scored 99.86% on the [0.5, 4] ERFC kernel, against 100% with double
   rounding. Divides and the final product are x87 too.
2. **Rational approximations, Horner in the raw variable.** R = N(x)/D(x), with both polynomials evaluated by
   Horner from the top coefficient, every step stored. There is no range reduction, no shifted variable and no
   Estrin scheme. Small-x erf uses z = fl(x*x) as the variable; the ERFC pieces use x itself.
3. **Degrees (n, n+1)** for the scaled function, with D's constant term normalised: 1 on [0.5, 4], and 128 (a
   power of two) on the tail. In the tail N0/D0 approximates 2/sqrt(pi), the asymptote of erfcx.
4. **Cody's break points, Excel's own coefficients.** ERF/ERFC split at 0.5 and 4, exactly as in Cody's CALERF
   (1969). But no published coefficient set matches (Cody, Cephes, Hart, Boost, fdlibm, SLATEC, the Numerical
   Recipes `erfccheb`, raw or rescaled). Excel ran its own fits.
5. **The coefficients come from a relative-error minimax (Remez) of the scaled function, but are not
   reproducible to the ulp.** The tail is a relative minimax (8,9) of erfcx on [4, 26.54], scaled by 128. A
   reconstruction lands within 2-14 ulps. On [0.5, 4], Excel's doubles sit 1.5E-9 to 1.6E-8 relative (up to 1.3E8
   ulps) from the 90-digit Remez solution. The shift runs along a near-null direction of the rational: the same
   function to about 1E-20, but different coefficients. Expect a generator to give you the structure and the degree,
   never the doubles. The doubles have to be measured.
6. **The outer function is assembled from other Excel functions, each rounded.** ERFC(x) = fl53(fl64(E*R)), where
   E is Excel's own EXP(-fl53(fl64(x*x))), a stored value. So EXP had to be modelled first. Find the composition
   before attacking the core.
7. **Branches switch on exact comparisons.** x = 4 takes the lower kernel. The small/large split is at
   0.5 + 2^-53. GAMMA.DIST shapes 0.5 and 1 are special-cased on exact equality, and 0.5 ± 1 ulp already takes the
   generic path. Probe both sides of every boundary at single-ulp resolution.
8. **Wrappers compose through exact identities, with x87 rounding at the glue.**
   - ERF = fl53(fl64(1 - ERFC)) above 0.5.
   - NORM.S.DIST(-y) = ERFC(fl53(fl64(y*fl(sqrt(0.5)))))/2: a multiply by sqrt(0.5), not a divide by sqrt(2).
   - CHISQ.DIST.RT(2z, 1) = ERFC(SQRT(z)).
   - POISSON is a CHISQ.DIST.RT composition.
   - ERF(a, b) is the exact difference of the two rounded values, with no worksheet cancellation snap.
9. **Underflow:** results below DBL_MIN are flushed to 0, not returned as subnormals.
10. **Excel's EXP is not correctly rounded.** It computes t = RN64(a*RN64(log2 e)), n = nint(t), F2XM1 on |f| <= 1/2,
    then reciprocates for f < 0 (model V6, numbers.md). It runs about 2% low by 1 ulp at large |a|. Its 93 remaining
    misses are F2XM1 near-ties that may depend on the CPU. Any kernel that calls exp, ln or pow probably inherits
    this style, so model Excel's elementary functions before the kernels built on them.

## 2. Probe design

The oracle is the bottleneck, so every cell should answer a question.

- **Long runs of consecutive doubles** (8,192 to 65,536 at a time): `x0 + k*ulp`. Rounding of stored
  intermediates shows up as deterministic sawtooth "lines" along a run. This is the main signal.
- **Irregular run starts.** Runs that all start on a dyadic grid (every one at m/128) create alias lattices:
  whole families of wrong coefficients reproduce the rounding pattern on that grid. Two peelers each found an
  exact-looking "pin" (z 57 and z 86) that was such an alias. Start runs at x0 = a + c*i + d with irrational-looking
  c and d (the `r11b` runs used 3.5 + 0.0103137*i + 0.0031415).
- **Runs just below powers of two** (x → 1⁻, 2⁻, 4⁻). There, a stored product's line frequency maps directly to its
  multiplicand's mantissa, and the stage values are simple sums of the coefficients.
- **Epsilon ladders**: short runs starting at x0 = b*(1 - eps) for eps = 2^-24, 2^-22, ..., 2^-4 below a branch
  end b. Phase offsets scale with eps, which unwraps each stage's deviation.
- **Always add the inner function's column.** Next to ERFC.PRECISE(x), add EXP(-(x*x)). Then E is known exactly,
  and you can recover the interval of possible R values from Excel's output by inverting the final rounding.
- **Side channels:** test wrappers that might expose an unrounded intermediate (1 - B, 2 - B ties, related
  distributions). For ERFC none did, but they confirmed the glue rounding.
- **Bisection probes** pin a binade crossing of an intermediate to the ulp, and the crossing pins an added constant.
- **Check every miss at ±1 ulp of the input before blaming the model.** Excel's xlsx number parser is not always
  correctly rounded: about one input in a million was read as the neighbouring double. A model miss that
  matches exactly at x ± 1 ulp is this quirk (numbers.md).
- **Size and logistics:** the excel-graph fill takes about a minute per 16,384-row case. Split runs into cases of
  16,384 rows, commit filled cases as they land, and tell waiting workers.

## 3. Analysis methods, roughly in the order that worked

1. **Composition first.** Find the identities and the outer formula (E*R) on a small probe, and confirm that R
   depends only on x.
2. **An exact verifier from day one.** Write a C scorer with `long double` and `volatile` stores, built with
   `gcc -O1 -mfpmath=387`. It should simulate each rounding exactly and report exact matches per x bin. Examples:
   `round9/mid_check.c` and `round7/erfc_tail_check.c`.
3. **Degree and structure by spectral tests, not least squares.** A least-squares fit forced into the wrong degree
   produces convincing artefacts. The (7,7) fit gave a "tiny top coefficient", "invisible top stages" and a "scale
   1.020495", and all three were false. What worked:
   - **Phase-free spectral overlap.** Compare the line spectrum of (data - smooth) with an exact stored-Horner
     simulation of a candidate, over hundreds of runs. Line frequencies depend only on the coarse intermediate
     values, so the right structure lines up even with coefficients 1E-12 off. Calibrate on a solved kernel: the
     tail scored 0.95 with its exact coefficients and 0.61 at 1E-12 off. On [0.5, 4] the (9,10) scored 0.68, and
     every other degree scored at chance (0.01).
   - **Per-stage matched filter over a scale scan.** All stages of the true chain light up coherently at one scale
     (combined z 23.5).
   - **An rms floor against parameter count**, calibrated with synthetic data from known kernels of each degree.
4. **The phase-lattice global solve, the decisive step.** For a stored product p = fl(s*x), the sawtooth phase of
   its rounding error is linear in the integer mantissa errors of the coefficients that feed s. Lock in each
   (run, stage) phase, then solve all coefficient errors at once as a closest-vector problem (fpylll LLL, about 850
   equations, a box of ±1E9 ulps). It took [0.5, 4] from 31% to 85.5% exact. Use a box wide enough for the
   near-null drift described in section 1, item 5.
5. **Top coefficients: chain solves and stage peeling.**
   - Near a branch end, product phases scale with eps → 0, so top-coefficient errors hide there. Recover them up
     the Horner recursion, ds_(k+1) = (ds_k - dc_k)/x, choosing the integer dc_k that makes the next product's
     phases coherent.
   - Peeling is top-down stage correlation: scan one coefficient and correlate the predicted rounding of its stage
     with the data residual.
   - Two independent methods agreeing to a few ulps is the confirmation.
6. **Interval inversion plus LP/MILP**, when stages are few. Invert each rounding into linear constraints on the
   coefficients. This solved small-x erf (p0..p2 and q0..q2 pinned to single doubles).
7. **Finish with exact 1-D and 2-D searches** on the last coefficients, scored by exact matches.

## 4. Traps

- **Exact-match rate is no objective until you are close.** Any smooth near-miss scores 22-40%. A model 1E-12 off
  scored the same as a random smooth model, and fewer roundings score higher regardless of truth. The rate rewards
  you only in the last few ulps. Use per-x-bin match rates to see where the error lives: a rate that falls with x
  points at the top coefficients.
- **Look-elsewhere.** Over 1E5 candidates, run-correlated noise reaches z 10-15. A single-stage hit at z 12 is not
  evidence. Require coherence across stages, or a hit in two or more disjoint x regions.
- **Grid aliases** (section 2): low bits right, high part wrong. The aliased pins had exactly the true low bits.
- **Don't centre searches on a generator.** Windows of ±1E4 ulps around the Remez solution were all empty, because
  the truth was about 1E8 ulps away.
- **Positive controls before trusting a method.** Generate synthetic data with an exact simulation of a known
  kernel, run the same pipeline, and confirm it recovers the truth, at the same point count and in the same x
  ranges. Every method that worked was validated this way first, and the false peaks were caught this way.
- **Published sources and memory are no evidence.** Ten-plus libraries were swept and none matched any kernel.
  Use them for structure hints (break points, degrees), never for coefficients.
- **Error-curve shape is a rounding fingerprint, not approximation error.** The "±3-ulp drift" in the ERFC tail
  was a staircase from binade crossings of Horner intermediates. Only exact simulation explains it.

## 5. Running it in parallel

- Give workers different methods, not slices of one method: generator reconstruction, spectral structure tests,
  phase-lattice solves, stage peeling. The breakthroughs came from cross-checks between them.
- When two workers disagree, have each run its test on the other's hypothesis before going further.
- The coordinator re-verifies every claimed set on the full committed data with its own scorer, and records
  negative results the same day.
- Commit probe data as it lands. Workers read it from the research branch.

## 6. Checklist for a new kernel

1. Search `docs/excel-semantics/` and this playbook. Find the function's identities with other Excel functions.
2. Probe the composition (include inner-function columns) and every boundary at 1-ulp resolution.
3. Model any Excel elementary function it calls (EXP, LN, POWER) to the bit.
4. Collect long runs: irregular starts, runs below powers of two, ladders below branch ends.
5. Build the exact x87 verifier.
6. Fix the structure (pieces, degree, normalisation) with calibrated spectral tests.
7. Get a generator start (relative minimax of the scaled function on the piece) for the structure only.
8. Run the phase-lattice solve with a wide box, then chain-solve or peel the top, then finish with exact searches.
9. Hold out data and check that misses are input misparses.
10. Record the quirks, the coefficients and the negatives in `docs/excel-semantics/`.

## 7. Open kernels

These use the same machinery and are still unsolved. Each has its own entries in `docs/excel-semantics/`:
- EXP's last 93 F2XM1 near-tie cells, possibly CPU-dependent.
- The NORM.S.INV centre (T-1746).
- BETA.DIST (T-1744).
- The generic-shape incomplete gamma and GAMMALN (statistics.md).
- The odd-degree-of-freedom chi-square paths.
