---
id: T-1928
title: "Worksheet EXP uses Excel's x87 model V6 (T-1778)"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1916]
---

## Goal
The worksheet `EXP(a)` returns Excel's own bits, which are not the correctly rounded exp: about 2.1% of
cells at large |a| are 1 ulp off it, mostly low.

## Context
- Model V6 (docs/excel-semantics/numbers.md, "EXP is an x87 2^(a*log2 e) evaluation, not correctly
  rounded"; reference docs/research/t1778/round8/exp_v6.py) matches 2,139,591 of 2,139,684 EXP cells of the
  T-1778 `kmyst-erfc-*` probes. The 93 misses are F2XM1 near-ties (2^|f| - 1 within 0.09 ulp of a 64-bit
  midpoint), possibly CPU-dependent.
- T-1916 implemented it as an internal helper, `crates/rustsheet-engine/src/x87_exp.rs` `excel_exp`, used
  only by the ERFC kernel. Its bulk check: 62,013 of 62,100 `EXP(A)` cells of `kmyst-erfc-r5expmap1/2` and
  7,234,707 of 7,235,446 `EXP(-(A*A))` cells (all misses are near-ties or the worksheet flush below).
- The worksheet EXP flushes a subnormal result to 0 (`kmyst-erfc-cutoff` column C: `EXP(-(x*x))` is 0 from
  x = 26.616 where the model gives 2.19E-308); the internal helper keeps gradual underflow.
- Every caller of `f64::exp` that stands for Excel's EXP inside another function (SINH/COSH/TANH, FISHERINV,
  GAMMA.DIST prefixes, the normal densities) should be checked against probes before it switches too.
- Evidence to copy: a curated subset of `kmyst-erfc-r5expmap1/2`, `kmyst-erfc-expcol` and the `C` columns of
  the `kmyst-erfc-*` runs on branch `agent/research/T-1778-kernel-map` (the full set is about 1.8 GB; never
  merge the branch).

## Acceptance criteria
- [ ] `fn_exp` uses `excel_exp` with the worksheet flush and overflow rules unchanged; a curated EXP probe
  subset (with provenance) is bit-exact except the listed V6 near-tie cells.
- [ ] `make eval-baseline` shows no regression (EXP, SINH/COSH/TANH, FISHERINV, power and exp-ln probes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 93 F2XM1 near-ties (an AMD F2XM1 hypothesis is untested; numbers.md).
