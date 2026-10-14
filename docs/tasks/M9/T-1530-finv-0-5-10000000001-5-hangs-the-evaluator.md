---
id: T-1530
title: "FINV(0.5,10000000001,5) hangs the evaluator: bound the F-distribution quantile for huge df"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
No F-distribution function hangs on a huge df: `FINV(0.5,10000000001,5)` and `FINV(0.5,2,10000000001)` return at once with Excel's result (a number or `#NUM!` per the oracle).

## Context
`probes-numeric/m9-legacy-finv` still times out (60s) in `make eval` after T-1518; the two cells above are the only slow ones (found by running each formula with a 5s limit). Cause: `fn_f_inv*` in `functions/statistical/fisher.rs` hands df to `statrs::FisherSnedecor::inverse_cdf`, which does not finish for a df around 1E10 (same failure as Student-t, T-1518). Probe Excel's upper df limit for F.INV/F.INV.RT/FINV/F.DIST* with the `excel-probe` skill (Student-t cut at 1E10 is in docs/excel-semantics/statistics.md), add a bounded quantile like `t_quantile` in `t_dist.rs`, and add a unit test (decision 0035).

## Acceptance criteria
- [ ] Both formulas above return within a bounded time; unit test covers them.
- [ ] `m9-legacy-finv` runs to completion in `make eval`.
- [ ] Excel's df cut for the F family is oracle-probed and recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probed Excel (case `t1530-f-df-cut`): either df above 1E10 is `#NUM!` for every F function (cdf, pdf, inverse, legacy names); recorded in statistics.md.
- `fisher.rs` rejects df outside 1..=1E10 and finds the quantile by bisection on the cdf when either df is 1E5 or more (`statrs` `inverse_cdf` does not finish there). Unit tests in `test_fn_f_df_limit.rs`.
- Eval: `m9-legacy-finv` now runs to completion (was a 60s timeout); `t1530-f-df-cut` 42 of 58 cells pass; no baseline change (probes-numeric has none).
- Follow-up T-1538: values at df 1E7 to 1E10 are off from Excel in the 6th digit (`statrs` cdf accuracy).
