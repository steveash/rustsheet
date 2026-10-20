---
id: T-1945
title: "EXP fast path that keeps model V6's bits (perf, T-1931)"
milestone: M9
lane: functions-math
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1931]
---

## Goal
Perf: the worksheet EXP and the ERFC kernel get Excel's model V6 bits at close to libm cost. Since T-1931,
`x87_exp::excel_exp` takes about 1.1 to 1.3 us per call against 6 to 14 ns for `f64::exp`. Almost all of that time is
the 26-term double-double `expm1` series in `two_to_the_minus_one`.

## Context
- Since T-0959 the operator `^` and POWER with a non-integer exponent (other than ±0.5) also call `excel_exp`:
  math_lift "POWER, per row" went from 80 ms to 214 ms and "POWER, lifted" from 0.85 s to 2.12 s (T-0959 Outcome).
- The T-1931 Outcome has the `crates/rustsheet/benches/math_lift.rs` numbers ("EXP, per row", "EXP, lifted").
- Only RN64(2^|f| - 1) has to be exact, and only where 2^|f| - 1 is near a 64-bit rounding midpoint. Elsewhere an
  approximation good to about 2^-80 rounds to the same 64 bits.
- Candidate approaches (review of PR #1132):
  - a cheap approximation to about 2^-80, falling back to the full series only near a midpoint (a Ziv-style
    rounding test);
  - a 2^(j/64) table that cuts the series to about 10 terms, mostly in plain f64.
- Removing the per-step divisions and enabling FMA were measured and do not help.
- The evidence that has to stay bit-exact: probes-numeric `t1931-exp-*` and `t1916-*`, plus the `x87_exp.rs` unit
  tests.

## Acceptance criteria
- [ ] `excel_exp` is at least 10x faster on a per-call kernel microbenchmark, and the math_lift EXP cases come
  within about 20% of the `f64::exp` numbers in T-1931's Outcome (the cases spend ~59 ms per row and ~582 ms
  lifted outside the kernel), with before and after numbers in the Outcome.
- [ ] The math_lift POWER cases come within about 20% of their libm-`exp` numbers in T-0959's Outcome.
- [ ] Every probes-numeric `t1931-exp-*` and `t1916-*` cell keeps its verdict, and so does every `power-*` case. A randomized test over a wide
  range of a compares the fast path with the full series.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F2XM1 near-tie residuals (T-1940).
