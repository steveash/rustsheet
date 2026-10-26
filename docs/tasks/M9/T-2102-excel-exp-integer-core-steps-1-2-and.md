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
- [x] The math_lift `EXP, per row` / `EXP, lifted` / `POWER` cases are within about 20% of a libm-`exp` build, measured
  on one machine.
- [x] A randomized test compares the new core with the current one over a wide range of a, and every probes-numeric
  `t1931-exp-*`, `t1916-*` and `power-*` cell keeps its verdict.
- [x] `make check` green; reviewers approve.

## Out of scope
- The F2XM1 near-tie residuals (T-1940).

## Outcome
- `x87_exp::exp_with` (model V6 steps 1, 2 and 4, and step 3's 1 + u and reciprocal) now works in 64-bit
  arithmetic, as the widths allow. Step 1 is one 64 x 64 multiply rounded inline (`round_to_64_bits`). Step 2 keeps
  f as a u64 magnitude and a sign. Step 3's RN64(1 + u) is a shifted, rounded u. The f < 0 reciprocal
  (`reciprocal`) is an f64 estimate of 2^127 / W, refined once from its exact i128 remainder and corrected by at most
  one unit, with no u128 division (a runtime call). Step 4 is a correctly rounded u64 to f64 conversion and two exact
  power-of-two scalings while the result is normal. `exp_with` is generic over step 3, so the estimate is a direct
  call instead of a function pointer.
- New double-level shortcut (`two_to_the_fraction`, on in `excel_exp_extended`, not in the series reference). The
  three RN64 roundings of step 3 leave w within 2^-63 * 2^f of 2^f (1.25 * 2^-64 for f > 0, about 1.96 * 2^-64 for
  f < 0). So when 2^f is more than 2^-60 (`DOUBLE_MIDPOINT_MARGIN`) from every double rounding midpoint, RN53(w) is
  the double nearest 2^f. A cheaper double-double 2^f decides that, and only then is the exact chain skipped:
  - 2^f comes from the same table reduction (`reduce_fraction`, `add_table_term`, now shared with
    `two_to_the_minus_one_estimate`), with expm1's tail in plain f64. Its error is about 2^-67;
    `fraction_estimate_error_bound` checks 2^-64 for 2^f, and for 2^-f through the `reciprocal_near_one` correction,
    on 156,000 points.
  - It falls back when the estimate is within the margin of a midpoint, or its head is 1. From the margin, about 2^-6
    of calls for 2^f in [1, 2) and 2^-5 in [0.5, 1) fall back (estimated, not counted).
  - It is needed for the target: the 64-bit integer core alone measured about 75 ns a call here.
- Bit identity:
  - `integer_core_matches_reference` keeps the old u128 core as a test-only reference (`exp_with_reference`). It
    compares 300,000 x 5 random arguments, each as a loaded double and as a full-width register, with and without
    the shortcut. The arguments are uniform in [-750, 750], at the underflow end [-745.2, -708.2], the overflow end
    [700, 710], and every binade from 2^-70, both signs. It also checks 20,000 arguments with the series step 3.
  - `reciprocal_matches_division` checks 1,000,000 random W and 2,000 at each end of [2^63, 2^64).
  - The T-1945 and T-2645 tests pass unchanged: `fast_path_matches_series`, `excel_exp_fast_path_matches_series`,
    `estimate_error_bound`, the 5,200 mpmath extended arguments and `excel_cells`.
  - probes-numeric was run with a `main` eval binary and with this branch's binary. Every case's verdicts and
    failing cells are identical in all 263 cases (298,103 of 611,860 cells pass on both). The 24 `t1931-exp-*`,
    `t1916-*` and `power-*` cases pass 60,851 of 60,951 cells on both.
- Per-call microbenchmark (`x87_exp::tests::per_call_cost`, release, 200,000 arguments in [-30, 30], medians of 9
  alternating runs on one machine). The T-1945 parent 34d61174 is rerun with the same loop:

  | | ns per call |
  |---|---|
  | parent 34d61174 (26-term series) | 844 |
  | main before this task (T-1945 estimate, u128 core) | 113 |
  | **this branch** | **54 (15.6x the parent)** |
  | libm `exp` | 1 |

- math_lift (`math_lift NAME --bench`, one case per process, medians of 11 runs). The libm build is this branch with
  `excel_exp` returning `a.exp()`, on the same machine:

  | case | this branch | libm-`exp` build |
  |---|---|---|
  | `EXP, per row` | 48.1 ms | 50.0 ms |
  | `EXP, lifted` | 473 ms | 464 ms |
  | `POWER, per row` (median of 4 processes) | 73 ms | 65 ms (+12%) |
  | `POWER, lifted` (median of 4 processes) | 735 ms | 673 ms (+9%) |
  | control `ABS, per row` | 31 ms | 24 ms (noise; ABS does not call EXP) |

  This shared machine is noisy (spreads of 10 to 30%), but every EXP and POWER case is within about 20% of libm.
- Eval: no change in any gated suite (`make check`). No Excel behaviour changed, so there is no excel-semantics entry.
- The tests moved to `x87_exp/tests.rs`, which keeps `x87_exp.rs` under the 1,000-line gate.
