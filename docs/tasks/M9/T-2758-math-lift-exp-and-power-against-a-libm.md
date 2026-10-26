---
id: T-2758
title: "math_lift EXP and POWER against a libm-exp build after T-2102's 81 ns excel_exp"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2102]
---

## Goal
Know whether the math_lift `EXP, per row`, `EXP, lifted`, `POWER, per row` and `POWER, lifted` cases are within about
20% of a libm-`exp` build, and close the gap if they are not (T-2102's second criterion, which it left unmeasured).

## Context
- T-2102 brought `x87_exp::excel_exp` from 952 to 81 ns a call (`x87_exp::tests::per_call_cost`, release, one
  machine; 11.8x the full-series parent) with the same bits, but did not run the math_lift bench.
- T-1945's estimate from the `ABS, per row` control plus 5 ns a libm call: a libm build is about 21 ms per 100,000 rows
  on its machine, so the per-row cases need about 50 ns a call or less to be within 20%; the lifted ones carry about
  420 ns a call of lifting overhead, so 81 ns is probably inside 20% there.
- Measure: `cargo bench -p rustsheet --bench math_lift -- NAME`, one case per process, medians of 11 runs, on main and on
  a scratch build whose `excel_exp` is `f64::exp` (never committed), same machine.
- If the per-row cases miss: the estimate in `two_to_the_minus_one_estimate` is latency-bound (about 40 ns, a chain of
  Dekker products); candidates are a larger table (smaller r, fewer exact terms) or a fixed-point u128 evaluation.

## Acceptance criteria
- [ ] The four cases' medians on main and on the libm scratch build, one machine, in the Outcome.
- [ ] If any case is more than about 20% slower than libm: a change that brings it within 20% with the same bits
  (`x87_exp` tests, including `integer_core_matches_the_reference`), or a decision record accepting the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F2XM1 near-tie residuals (T-1940).
