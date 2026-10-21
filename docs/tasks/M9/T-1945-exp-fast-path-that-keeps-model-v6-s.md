---
id: T-1945
title: "EXP fast path that keeps model V6's bits (perf, T-1931)"
milestone: M9
lane: functions-math
status: done
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
Rescoped at review (PR for T-1945): the fast path landed at 7.8x per call, and the remaining cost is the model's
integer core, outside the series this task targets. The original targets, at least 10x per call and the math_lift EXP
and POWER cases within about 20% of libm, moved unchanged to **T-2102**.
- [x] A fast path that keeps model V6's bits replaces the 26-term series on all but near-midpoint calls, with before
  and after numbers (per-call microbenchmark and the math_lift EXP and POWER cases) in the Outcome.
- [x] Every probes-numeric `t1931-exp-*` and `t1916-*` cell keeps its verdict, and so does every `power-*` case. A
  randomized test over a wide range of a compares the fast path with the full series.
- [x] `make check` green; reviewers approve.

## Out of scope
- The F2XM1 near-tie residuals (T-1940).

## Outcome
- `x87_exp::two_to_the_minus_one` (F2XM1's RN64(2^f - 1)) now starts from a table-driven estimate: f = j/64 + r,
  2^f - 1 = E + e + E e with E = 2^(j/64) - 1 from a 33-entry double-double table and e = expm1(r ln 2),
  |r ln 2| <= 0.0055, whose terms through y^4/24 are kept exactly with Dekker products and the rest summed in plain
  f64 (about 2^-85 relative). A Ziv-style test takes the estimate's rounding unless it lies within 2^-9 half-units of
  the 64-bit grid (at least 2^-74 of the value) of a rounding midpoint; only then does the old 26-term series
  (`two_to_the_minus_one_series`, unchanged) decide, about 1 call in 512. The model itself (steps 1, 2 and 4) is
  unchanged; `exp_with` lets the tests run it on either step 3.
- Also on the hot path: `arithmetic::round_to_bits` and step 2's FRNDINT round up with integer bit operations instead of
  `||`/`&&` (the round-up is data-dependent and the branch mispredicted half the time, about 10 ns a call), the fast
  path uses Dekker's `TwoProduct` instead of `mul_add` (a runtime-dispatched call without the `fma` target feature)
  and an i64 split instead of i128 <-> f64 conversions (compiler-builtins calls). trig.rs's private `two_sum` and
  `fast_two_sum` now call the `x87_exp` ones.
- Bit identity: `fast_path_matches_series` (every drop in 53..=130, about 320,000 magnitudes of random bit length,
  plus each table node and its neighbours), `excel_exp_fast_path_matches_series` (600,000 arguments: uniform in
  [-750, 750] and random significands at every binade from 2^-65, both signs) and `estimate_error_bound` (the estimate
  is within 2^-80 of the series sum on 156,000 more points) all pass. probes-numeric with the parent's and this
  branch's eval binary: identical results in all 153 cases (73,783 of 80,719 cells pass on both; the 24 `t1931-exp-*`, `t1916-*` and
  `power-*` cases pass 60,844 of 60,951 on both).
- Per-call microbenchmark (`x87_exp::tests::per_call_cost`, release, 200,000 arguments in [-30, 30], one machine):
  `excel_exp` **971 ns -> 125 ns (7.8x)**; libm `exp` 5 ns. In a standalone copy, about 55 ns of the rest is the
  estimate and about 50 ns the model's u128 integer core (steps 1, 2 and 4, plus the f < 0 reciprocal's u128
  division), which this task did not touch.
- math_lift (`cargo bench -p rustsheet --bench math_lift -- NAME`, one case per process, medians of 11 runs, parent
  34d61174 and this branch on the same machine):

  | case | parent (V6, full series) | this branch |
  |---|---|---|
  | `EXP, per row` | 133.5 ms | 40.8 ms |
  | `EXP, lifted` | 1.344 s | 0.543 s |
  | `POWER, per row` | 138.1 ms | 44.7 ms |
  | `POWER, lifted` | 1.366 s | 0.544 s |
  | control `ABS, per row` | 20.1 ms | 18.8 ms |

  This machine is about 3x faster than T-1931's on the per-row overhead (ABS 20 ms against 59 to 65 ms), so the
  T-1931/T-0959 libm numbers do not transfer. Estimated from the ABS control plus 5 ns a call, a libm-`exp` build
  here would be about 21 ms per row: the per-row EXP case is still about 2x that, and the lifted ones carry about
  125 ns a call more than libm.
- **The original targets are not met:** 7.8x per call against 10x, and math_lift is not within about 20% of libm. The remaining cost is mostly outside the series this task targeted. **T-2102** (functions-math,
  opus) owns the integer core and the rest of the gap, with these numbers as its baseline.
- eval: no change in any gated suite (`make check`); no probes-numeric verdict changed.
