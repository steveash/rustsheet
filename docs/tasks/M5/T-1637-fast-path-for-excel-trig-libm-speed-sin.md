---
id: T-1637
title: "Fast path for Excel trig: libm-speed SIN/COS with a rounding test, double-double only when needed"
milestone: M5
lane: perf
status: in-progress
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
- [ ] A release microbenchmark of `excel_sin_cos` and `excel_tan` over angles across [-1e3, 1e6], before/after, in the Outcome.
- [ ] The fast path equals the double-double series bit for bit on a large random sweep (a unit test like `mixed_precision_series_equals_the_full_double_double_series`), and probes-numeric/m9-trig-reduction stays 666/666.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the Excel model (shift, SEC/CSC/COT as 1 over the rounded value).
