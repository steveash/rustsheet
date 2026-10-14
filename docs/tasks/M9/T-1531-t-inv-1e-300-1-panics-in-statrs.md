---
id: T-1531
title: "T.INV(1E-300,1) panics in statrs inverse_cdf: reject or guard tiny p before the Student-t quantile"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
No Student-t function panics on a tiny probability: `=T.INV(1E-300,1)` returns Excel's result (number or `#NUM!`, per the oracle) without panicking (directive 5).

## Context
The T-1518 performance review ran statrs 0.18.0 directly: `StudentsT::inverse_cdf` unwraps `XOutOfRange` in `inv_beta_reg` at p=1E-300 for df=1, 1.5 and 99999. The engine has no `catch_unwind`. `t_quantile` in `functions/statistical/t_dist.rs` (shared by T.INV, T.INV.2T, CONFIDENCE.T) is the single place to guard. Probe Excel for the smallest p it accepts (excel-probe), then reject or clamp before calling statrs; the same panic may exist in other statrs-based quantiles (list them in the Outcome).

## Acceptance criteria
- [ ] `T.INV(1E-300,1)`, `T.INV.2T` near p=2 and `CONFIDENCE.T` at tiny alpha do not panic; unit tests cover them.
- [ ] Excel's behavior at tiny p is oracle-probed and recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probed Excel (`probes-numeric/t1531-t-tiny-p`, 105 cells): the Student-t quantile is finite, never an error, for p down to 1E-300; recorded in statistics.md.
- `statrs` `StudentsT::inverse_cdf` panics from p about 1E-200 and is already wrong for small df well before (`T.INV(1E-10,1)` = -8.9E7, Excel -3.18E9). `t_quantile` now takes a closed-form tail asymptotic where x = df/(df+t^2) < 1E-8, `statrs` where it is bit-exact with Excel, and a bounded bisection otherwise. `t_upper_quantile` serves T.INV.2T and CONFIDENCE.T, whose `1 - p/2` rounds to 1 for a tiny p.
- Other statrs quantiles that may panic the same way: the F quantile was bounded by T-1530; the remaining `inverse_cdf` callers (`normal.rs`, `log_normal.rs`, `chisq.rs`, `gamma.rs`, `beta.rs`) are listed in follow-up T-1543.
- Eval: `t1531-t-tiny-p` 7 of 105 cells bit-exact (before: the engine panicked); no regression in any gated suite, no baseline change.
- Follow-up T-1542 (last digit, df >= 99999 at p <= 1E-200, T.INV.2T for p in (1,2)).
