---
id: T-1723
title: "CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX: size, single-point and text-argument errors and large-offset data as Excel (T-1620 probes)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
CORREL, PEARSON, RSQ, SLOPE, INTERCEPT and STEYX return Excel's errors for mismatched sizes, a single point and a text argument, and stay accurate on data with a large offset.

## Context
Found by T-1620 (probes/regression-pairs-a, regression-pairs-b; `./target/release/rustsheet-eval show probes/regression-pairs-a.case.json`, excel-graph 2026-10-16). Differences, same for all six unless noted:
- unequal sizes with all numbers (`CORREL(A1:A5,B1:B6)`, `CORREL({2,4,5},{1,2})`) and a text scalar argument (`CORREL(A1:A6,"a")`) are `#N/A` in Excel, `#VALUE!` in ours (T-1292 covers the text-in-range variant).
- a single point or scalar pair (`CORREL(5,5)`, `CORREL(A1,B1)`) is `#DIV/0!` in Excel, `#VALUE!` in ours.
- data with a large offset (`I1:I6` = 10000000001, 10000000002, ...): CORREL(I1:I6,B1:B6) is 0.9402561526801988 in Excel, `#DIV/0!` in ours (ours loses the variance to cancellation; STEYX gives 0.8223080178319075 against 0.822308017832176); compute the sums of squares about the means.
- last-digit differences (`CORREL(A1:A6,C1:C6)` 0.9999999999999998 in Excel, 1.0 in ours; INTERCEPT, STEYX, SLOPE similar): match Excel's accumulation order if the probes pin one down.
Overlap: T-1292 (unequal sizes with text) and T-1707 (CORREL, PEARSON, RSQ unequal sizes and the last digit) cover part of the size and ULP cells; take those first or fold them in, and close whichever is then redundant. What is new here: the single-point and scalar `#DIV/0!` cases, the text scalar `#N/A`, SLOPE, INTERCEPT and STEYX on all-number size mismatches, and large-offset data.
Record any new quirk in docs/excel-semantics/statistics.md.

Overlaps T-1401 (scalar or single cell as a one-value array: the `#DIV/0!` single-point cases) as well as T-1292 and T-1707. The exact formula for r and r² on large-offset data is open (a two-pass sum of squares about the means reproduces Excel's bits for SLOPE, INTERCEPT, STEYX and FORECAST, not CORREL). Also covers STEYX `#DIV/0!` with two points, zero-variance known_y (SLOPE 0, STEYX 0, INTERCEPT the mean, correlations `#DIV/0!`) and RSQ above 1 (`RSQ(J1:J6,B1:B6)` 1.0000000000000004). STEYX large-offset: Excel 0.822308017832176, ours 0.8223080178319075.

## Acceptance criteria
- [ ] The mismatching non-ULP cells of probes/regression-pairs-a and -b for these six functions pass and leave eval/baseline/probes.txt smaller.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing further inputs (file a probe task instead).

## Handoff (resumed)
Not started in code by the sonnet worker (2026-10-19). The task overlaps T-1292, T-1707 and T-1401 (same functions, same
error paths) and its large-offset/ULP cells need numeric fitting of Excel's accumulation order, which is beyond the sonnet
tier. Suggested order for the opus run: land T-1707 first (size `#N/A`), then this task's single-point `#DIV/0!` and
text-scalar `#N/A` cells, then the large-offset formula for r and r^2 (open, per the Context above).

## Outcome
CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX and FORECAST now share `PairedMoments` (correl.rs): two-pass sums about the
means, then Excel's formulas fitted to the bit on every number cell of probes/regression-pairs-a/-b: r =
(Sxy/n)/(sqrt(Sxx/n)*sqrt(Syy/n)) for CORREL/PEARSON, the same over n - 1 squared for RSQ, SLOPE Sxy/Sxx, INTERCEPT
ȳ - SLOPE*x̄, STEYX sqrt((Syy - Sxy*Sxy/Sxx)/(n-2)). A typed text or boolean argument is one value with no number (new
probe probes/regression-pairs-text-scalar, 72 cells, excel-graph), and CHISQ.TEST of a single value is `#N/A`
(probes-numeric/t1049-chisqtest A36). Eval: probes known failures 1948 -> 1882 (regression-pairs-a and
regression-forecast fully pass; pairs-b left with COVAR only, T-1724), ironcalc-calc 1057 -> 1056. Folds in T-1707 and
T-1725 (closed). Follow-up: T-2080 (STEYX on a near-exact fit).
