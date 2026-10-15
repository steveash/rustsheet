---
id: T-1543
title: "Sweep every statrs inverse_cdf caller for a panic at a tiny probability (normal, lognormal, chi-square, gamma, beta)"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
No `statrs` `inverse_cdf` call in the engine can panic or hang on a tiny or extreme probability (directive 5): each caller returns Excel's value or `#NUM!` for `1E-300` and `1-1E-16`.

## Context
T-1531 found that `StudentsT::inverse_cdf` panics from p about 1E-200 and is inaccurate well before. Unchecked callers (from the T-1531 architecture review): `normal.rs` (NORM.INV, NORM.S.INV, LOGINV-style, lines ~99, 174, 235), `log_normal.rs:107`, `chisq.rs:184,249`, `gamma.rs:151`, `beta.rs:202`; `rg 'inverse_cdf' crates/rustsheet-engine/src` finds them all. Probe each with the `excel-probe` skill (tiny p and 1-1E-16, small and large parameters), call it directly under `catch_unwind` in a scratch test to find the panic range, guard it, add unit tests and record Excel's behavior in statistics.md. Student-t is done in T-1531; the F quantile in T-1530.

## Acceptance criteria
- [ ] A unit test per caller covers a probability that used to panic (or the scratch sweep shows none does) and a probes-numeric case holds the Excel values.
- [ ] Excel's behavior at tiny p is recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Sweep (scratch test, removed) of NORM.INV, NORM.S.INV, LOGNORM.INV, CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, CONFIDENCE.NORM at p from 1E-300 to 1-1E-10 found no panic or slow case; BETA.INV panicked from p 1E-200 and hung at (1E-100,1000,1) and for shapes of 1E8 or more (F.INV panics too: T-1530). BETA.INV now uses `statrs` only for shapes up to 1E5 and p of 1E-15 or more and bounded bisection (`bisect_unit_interval`) otherwise; unit test `test_fn_beta_inv_tiny_probability_is_bounded`. Excel values recorded in docs/excel-semantics/statistics.md and probes-numeric case `t1543-inverse-cdf-tiny-p` (oracle-filled). Eval: no regressions, no baseline change. Tail value mismatches for CHISQ/GAMMA: T-1633.
