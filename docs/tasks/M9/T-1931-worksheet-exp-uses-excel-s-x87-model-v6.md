---
id: T-1931
title: "Worksheet EXP uses Excel's x87 model V6 (T-1778)"
milestone: M9
lane: functions-math
status: done
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
  x = 26.616 where the model gives 2.19E-308); the internal helper keeps gradual underflow. (Later shown to be a
  Graph artifact, decision 0042: the worksheet keeps the subnormal; see acceptance criterion 1.)
- Every caller of `f64::exp` that stands for Excel's EXP inside another function (SINH/COSH/TANH, FISHERINV,
  GAMMA.DIST prefixes, the normal densities) should be checked against probes before it switches too.
- Evidence to copy: a curated subset of `kmyst-erfc-r5expmap1/2`, `kmyst-erfc-expcol` and the `C` columns of
  the `kmyst-erfc-*` runs on branch `agent/research/T-1778-kernel-map` (the full set is about 1.8 GB; never
  merge the branch).

## Acceptance criteria
- [x] `fn_exp` uses `excel_exp`, keeping subnormal results (the "flush" above was the Graph artifact of decision
  0042) and the overflow rule unchanged; a curated EXP probe subset (with provenance) is bit-exact except the
  listed V6 near-tie cells.
- [x] `make eval-baseline` shows no regression (EXP, SINH/COSH/TANH, FISHERINV, power and exp-ln probes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 93 F2XM1 near-ties (an AMD F2XM1 hypothesis is untested; numbers.md).

## Outcome
- `fn_exp` calls `x87_exp::excel_exp` (model V6) and nothing else. The "worksheet flush" in the Context is a
  Graph artifact: V6 keeps subnormals, as Excel does (probes/subnormal-function-results C4 and C6 stay FALSE), and the
  18 `kmyst-erfc-cutoff` C cells that read 0 are kept subnormals reported as 0.0 (decision 0042). Overflow is still
  infinite, so `#NUM!`. EXP is entered in the kernel-work table with its fixed cap.
- Evidence: probes-numeric `t1931-exp-map` (5,990 EXP cells), `t1931-exp-random` (7,084), `t1931-exp-gauss`
  (8,096 `EXP(-(x*x))`), `t1931-exp-neg` (1,024 `EXP(-z)`) and `t1931-exp-ties` (87): 44,562 cells, curated from
  `kmyst-erfc-r5expmap1/2`, `-expcol`, `-driftmap` and `-chiruns1` at 59c2c3724a3c and filled fresh from excel-graph
  (2026-10-19), which returned the research run's value at all 22,281 formula cells. The engine matches all
  22,194 non-tie cells (libm's `exp`: 16,183); the 87 ties are 1 ulp misses (+1 at 47, -1 at 40), owned by T-1940.
  libm matches 43 of the ties, so probes-numeric goes 65,579 -> 71,547 passing cells (+6,011 non-tie, -43 tie).
- eval: no change in any gated suite (ironcalc-calc 1144, poi 481, probes 2133, fixtures 9 known failures; no
  regressions, no fixes), and no other probes-numeric case changed.
- SINH/COSH/TANH/FISHERINV stay on libm: switching them to `excel_exp` changed no probed cell (numbers.md), so there
  is no evidence either way and no follow-up switch task.
- Performance (review of PR #1132). V6 is slower than libm. `excel_exp` costs about 1.1 to 1.3 us per call against
  6 to 14 ns for `f64::exp`, and nearly all of that is the 26-term double-double series in `two_to_the_minus_one`.
  New cases in `crates/rustsheet/benches/math_lift.rs` measure it (release, `cargo bench -p rustsheet --bench
  math_lift -- NAME`, one case per process, medians of 11 runs, two runs each of the parent with `f64::exp` and of
  this branch):

  | case | before (`f64::exp`) | after (V6) |
  |---|---|---|
  | `EXP, per row`: 100,000 cells `=EXP(A{r}/100000)` | 59 to 60 ms | 174 to 199 ms |
  | `EXP, lifted`: `=SUM(EXP(A1:A1000000/1000000))` | 582 to 608 ms | 1.69 to 1.70 s |
  | control `ABS, per row` / `ABS, lifted` | 59 to 65 ms / 459 to 464 ms | 47 ms / 444 to 451 ms |

  That is about 2.9x on EXP-only sheets, or roughly 1.1 us more per EXP evaluation. The ABS controls do not move.
  It is accepted for parity (directive 1): libm's exp misses 6,011 of the 22,194 curated Excel cells, and V6 misses
  none of them. The cost is a fixed, allocation-free amount per call (the kernel-work table entry), so it cannot
  grow with the input. T-1945 (functions-math, perf) owns a fast path that keeps V6's bits. The reviewer found that
  dropping the per-step divisions or enabling FMA does not help.
- Follow-ups: T-1940 (the F2XM1 near-tie residuals), T-1945 (a V6 fast path, perf), T-1944 (probe EXP's overflow
  edge near 709.78, `EXP(710)`, and the underflow edge near -745.13).
