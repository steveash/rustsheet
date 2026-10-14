---
id: T-1538
title: "F-distribution quantile and cdf at huge df (1E7 to 1E10) differ from Excel in the 6th digit"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
F.INV, F.INV.RT, FINV, F.DIST and FDIST at a huge df (1E7 up to 1E10) match Excel's values to the last digits, e.g. `F.INV(0.5,1E10,1E10)` = 1 and `F.INV(0.5,10000000000,5)` = 1.1490395521130337.

## Context
T-1530 bounded the quantile (bisection on the `statrs` cdf, `f_quantile` in `functions/statistical/fisher.rs`) so it no longer hangs, but `statrs`' cdf is only good to about 1E-6 at huge df (same as Student-t, see T-0967). Failing cells: 16 of 58 in probes-numeric `t1530-f-df-cut` (e.g. A1 gives 1.149036767064711), and `F.INV(0.5,1E10,1E10)` gives 1.0000025. The pdf cells A9/A10 (`F.DIST(...,FALSE)` at df 1E10) are `#NUM!` for us, a type mismatch, not precision. Excel's `F.INV` and `F.INV.RT(1-p)` also differ at df 1E10 (A1 vs A3, A2 vs A4; statistics.md), so the two cannot share one quantile path. Needs a fit or an own incomplete-beta kernel; see docs/excel-semantics/statistics.md (F-distribution df limit).

## Acceptance criteria
- [ ] `t1530-f-df-cut` cells A1-A14 and A57-A58 pass bit-exact or the residual is recorded as a DEVIATION.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
