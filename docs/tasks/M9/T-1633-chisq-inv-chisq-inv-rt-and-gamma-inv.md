---
id: T-1633
title: "CHISQ.INV, CHISQ.INV.RT and GAMMA.INV tails: match Excel at tiny and near-1 probability (T-1543 probe)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
CHISQ.INV, CHISQ.INV.RT and GAMMA.INV match Excel's oracle values for p from 1E-300 to 1-1E-16 (no `#NUM!` where Excel has a value, no wrong digits in the tails).

## Context
probes-numeric/t1543-inverse-cdf-tiny-p (oracle-filled, T-1543) fails at Calc!A25-A28, A29-A33, A35-A36, A41-A46, A49-A61, A65-A66 (run `rustsheet-eval show probes-numeric/t1543-inverse-cdf-tiny-p.case.json`). `statrs`' gamma `inverse_cdf` is wrong in the upper tail (CHISQ.INV(1-1E-16,50) = 179.80, Excel 180.68) and fails in the lower tail; the fix is likely bisection on the exact cdf as in beta.rs `beta_quantile`/special.rs `bisect_unit_interval`. Details in docs/excel-semantics/statistics.md, T-1543 entry.

## Acceptance criteria
- [ ] The listed cells of t1543-inverse-cdf-tiny-p pass (within the suite's numeric tolerance) and ordinary-p cases do not regress.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
