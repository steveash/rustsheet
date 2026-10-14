---
id: T-1529
title: "T.INV(1E-300,1) panics in statrs inverse_cdf: reject or guard tiny p before the Student-t quantile"
milestone: M9
lane: functions-stat
status: todo
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
