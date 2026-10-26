---
id: T-2102
title: "excel_exp integer core: steps 1, 2 and 4 of model V6 at a few ns (T-1945 follow-up)"
milestone: M9
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1945]
---

## Goal
Perf: `x87_exp::excel_exp` reaches the 10x per-call speed-up T-1945 set (about 97 ns or less, from 971 ns on the
T-1945 machine) and returns the same bits.

## Context
- T-1945 replaced the 26-term F2XM1 series with a table-driven estimate and a Ziv rounding test: 971 -> about 125 ns a
  call (`x87_exp::tests::per_call_cost`, release). Measured in a standalone copy, about 50 ns of the rest is the
  model's u128 integer pipeline, which is outside the estimate: step 1 (RN64 of the 117-bit `a * RN64(log2 e)`), step 2
  (FRNDINT and the exact fraction), step 3's RN64(1 + u) and the reciprocal for f < 0 (one u128 division, about 10 ns),
  and step 4 (`arithmetic::round_extended_to_double`). The estimate itself is about 55 ns, latency-bound.
- Candidates: u64 arithmetic where the widths are known (t, w and the 53-bit rounding all fit 64 bits), a reciprocal
  from an f64 estimate plus one exact u128 correction, and a shorter dependency chain in the estimate (Estrin for
  its f64 tail, E e started before e is complete).
- Bit identity is checked by `x87_exp` tests `fast_path_matches_series` and `excel_exp_fast_path_matches_series`, plus
  probes-numeric `t1931-exp-*`, `t1916-*` and `power-*`. A rewritten core needs its own randomized comparison with the
  current one, since those tests share it.

## Acceptance criteria
- [x] `per_call_cost` shows `excel_exp` at least 10x faster than the T-1945 parent's 971 ns on the same machine (rerun
  the parent to compare), with before and after numbers in the Outcome.
- [ ] The math_lift `EXP, per row` / `EXP, lifted` / `POWER` cases are within about 20% of a libm-`exp` build, measured
  on one machine.
- [x] A randomized test compares the new core with the current one over a wide range of a, and every probes-numeric
  `t1931-exp-*`, `t1916-*` and `power-*` cell keeps its verdict.
- [x] `make check` green; reviewers approve.

## Out of scope
- The F2XM1 near-tie residuals (T-1940).

## Outcome
- `x87_exp::exp_with` (steps 1, 2 and 4 of model V6) now works on u64 halves: step 1 rounds the top 64 bits of the
  normalised `significand * RN64(log2 e)` product (`round_top_64`), step 2's FRNDINT and the fraction's magnitude are u64
  shifts, 1 + u is one u64 rounding, and step 4 is `round_64_to_double` (the 64-bit case of
  `round_extended_to_double`, with the carry folded into the exponent field). The f < 0 reciprocal keeps one u128
  division but drops the second (`%`): the remainder is `2^127 - q w`. One `round_up` helper does the four
  ties-to-even round-ups without branches. An f64-estimate reciprocal was tried and was slower (a `roundeven` libcall,
  then two dependent f64 divisions).
- The estimate's dependency chain is shorter, same accuracy (worst error on the `estimate_error_bound` sweep
  2^-86.6, budget 2^-74): y = r ln 2 comes from one 64 x 128-bit integer product (`times_ln_2`) instead of an exact
  double-double r and a Dekker product; t = yh^3/6 is p * (yh/6) with yh/6 formed alongside p; and the table step is
  E + e * 2^(j/64) (one product and two FastTwoSums) instead of E + (e + E e). The Ziv test takes one nearest-integer
  instead of two (the same condition, exactly).
- Bit identity: the new `integer_core_matches_the_reference` compares the u64 core with the T-1945 u128 core (kept in
  the tests as `exp_reference`) on 1.5 million doubles (random bit patterns, so every exponent, the range guards and
  subnormal results; uniform in [-750, 750] and [-30, 30]; random significands at every binade from 2^-70) and 600,000
  full-width extended registers, all equal. `fast_path_matches_series`, `excel_exp_fast_path_matches_series`,
  `estimate_error_bound` and the 5,200-row mpmath fixture pass unchanged. probes-numeric with an eval binary built
  before this change and this branch's: all 263 cases identical cell for cell (298,103 of 611,860 cells
  pass on both; the 28 `t1931-exp-*`, `t1916-*` and `power-*` cases pass 69,439 of 77,391 on both).
- Per call (`x87_exp::tests::per_call_cost`, release, 200,000 arguments in [-30, 30], one machine, a 2.1 GHz Xeon VM):
  before this change `excel_exp` 137 ns and the series path (the T-1945 parent's whole algorithm, full F2XM1 series
  on the old core) 952 ns; after it, `excel_exp` **81 ns** (two runs, 81 and 81), **11.8x** the parent's 952 ns.
- **Not measured: the math_lift criterion** (the four EXP and POWER cases against a libm-`exp` build). T-1945's
  estimate puts the per-row cases' 20% bound at about 50 ns a call, so they probably still miss; the lifted ones
  probably pass. Filed **T-2758** (functions-math, opus) to measure and, if needed, close it.
- eval: no change in any gated suite (`make check`); no probes-numeric cell changed.

