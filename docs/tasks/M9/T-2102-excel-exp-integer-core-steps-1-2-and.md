---
id: T-2102
title: "excel_exp integer core: steps 1, 2 and 4 of model V6 at a few ns (T-1945 follow-up)"
milestone: M9
lane: functions-math
status: todo
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
- [ ] `per_call_cost` shows `excel_exp` at least 10x faster than the T-1945 parent's 971 ns on the same machine (rerun
  the parent to compare), with before and after numbers in the Outcome.
- [ ] The math_lift `EXP, per row` / `EXP, lifted` / `POWER` cases are within about 20% of a libm-`exp` build, measured
  on one machine.
- [ ] A randomized test compares the new core with the current one over a wide range of a, and every probes-numeric
  `t1931-exp-*`, `t1916-*` and `power-*` cell keeps its verdict.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F2XM1 near-tie residuals (T-1940).
