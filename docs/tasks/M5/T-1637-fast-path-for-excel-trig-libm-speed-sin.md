---
id: T-1637
title: "Fast path for Excel trig: libm-speed SIN/COS with a rounding test, double-double only when needed"
milestone: M5
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
SIN, COS, TAN, COT, SEC and CSC cost close to libm per call while staying bit-identical to the current Excel model.

## Context
T-1563 made `functions/math_and_trigonometry/trig.rs` reduce by the x87 π₆₄/2 and sum the Taylor series in double-double (docs/excel-semantics/numbers.md "Trig of a huge angle"). That costs about 150 ns per call against libm's 10 ns (release, baseline x86-64; about 90 ns with inline FMA), mostly the ~23 `mul_add` calls of the five double-double Horner steps. Standard remedy: evaluate a cheaper approximation with a known error bound (e.g. a double polynomial plus the r_lo and reduction-shift corrections) and fall back to the double-double series only when the result is too close to a rounding boundary (Ziv's rounding test). Cheaper first step (T-1563 performance review): SIN/COS/SEC/CSC need only one of the two series once the quadrant `n` is known, so split single-function entry points (about 2x); TAN/COT need both. Benchmark both the baseline and an FMA target and say which counts.

## Acceptance criteria
- [x] A release microbenchmark of `excel_sin_cos` and `excel_tan` over angles across [-1e3, 1e6], before/after, in the Outcome.
- [x] The fast path equals the double-double series bit for bit on a large random sweep (a unit test like `mixed_precision_series_equals_the_full_double_double_series`), and probes-numeric/m9-trig-reduction stays 666/666.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the Excel model (shift, SEC/CSC/COT as 1 over the rounded value).

## Outcome
`trig.rs` now sums each series twice when it has to: a fast sum first (sine: 2 outer Horner steps
in double-double, cosine: 3; the next step's `1 − t` kept exact as a double-double, the rest in
doubles), then Ziv's rounding test against a proved relative error bound (2^-63 sine, 2^-66
cosine; proof in `Series::fast`, margins 3.5x and 4.6x; a sweep measures 2^-65.8 and 2^-69.1).
Only a result whose bound straddles a rounding boundary (about 1 call in 500) runs the old full
series, so every result is the old one bit for bit. SIN/COS/SEC/CSC (`excel_sin`, `excel_cos`)
run only the series their quadrant needs; TAN/COT run both fast sums and test the quotient
against the sum of the two bounds. The fast sums are inlined so TAN's two series interleave.
Tests: the 200,000-angle sweep is now 400,000 and checks `excel_sin`, `excel_cos`,
`excel_sin_cos` and `excel_tan` against the full double-double series; a new test keeps the fast
sums within a quarter of their bounds, and tiny/subnormal angles are pinned.

Release microbenchmark, 2M uniform angles in [-1e3, 1e6], best of 7 (this container is about 2.3x
slower than the T-1563 machine: libm sin 23 ns here vs 10 ns there):

| per call | baseline x86-64 before → after | x86-64-v3 (FMA) before → after |
|---|---|---|
| SIN (`excel_sin`) | 324 → 86 ns | 275 → 63 ns |
| COS (`excel_cos`) | 324 → 88 ns | 275 → 63 ns |
| `excel_sin_cos` (both, IM*) | 322 → 139 ns | 268 → 102 ns |
| TAN/COT (`excel_tan`) | 327 → 128 ns | 269 → 89 ns |
| libm sin / tan | 23 / 29 ns | 23 / 29 ns |

The baseline target is the one that counts (release builds and CI do not enable FMA, so every
`mul_add` is a call into glibc's `fma`). A Dekker split instead of `mul_add` was tried on
baseline and was slower (166 vs 129 ns at that point), so it was dropped. Bit-identity on the
2M-angle benchmark set: 0 mismatches for all four functions. Eval delta: none (no result
changes; probes-numeric/m9-trig-reduction stays 666/666).
