---
id: T-3369
title: "Probe the M9 statistical array-parameter argument kinds no Excel cell covers yet: empty text and an error as the data array of 37 functions (CORREL, PEARSON, SLOPE, PERCENTILE, LARGE, ...)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every `array`-mode parameter of the M9 statistical functions below has at least one Excel-backed cell for each
argument kind the report's argument coverage expects, so the report shows `covered == expected` for them.

## Context
Gap triage 2026-10-31c (`make eval` on origin/main 0c917f9fe; report.json `argument_coverage`, decision 0031 evidence
gaps; decision 0094 asks for each argument mode to be covered). `scripts/exit_owners.py --all-off` shows no visibly-off
cell without an owner, so argument kinds are the evidence gap left.
T-3129 (the M9 aggregate, test and regression probe) left `array`-mode parameters out because it took their coverage
to be unclassified by kind. It is classified: `crates/rustsheet-eval/src/argument_coverage.rs` `expected_classes`
expects all 11 kinds of an `array` parameter, and `classify_value` classifies a literal or single-cell-reference argument
passed there. So the array parameters below show at most 9 of 11 kinds: no Excel cell passes empty text (`""`) or an
error as the whole argument. A multi-cell range holding `""` or an error does not count. The argument itself must be
the literal or a single-cell reference to a cell holding it.

These parameters have no Excel cell of the listed kinds in any suite (catalog `prior` 0.5 to 0.05,
as `FUNCTION.parameter: kinds`):
- LARGE.array, SMALL.array: empty text, error.
- PERCENTILE.array, PERCENTILE.EXC.array, PERCENTILE.INC.array, QUARTILE.array, QUARTILE.EXC.array,
  QUARTILE.INC.array: empty text, error.
- CORREL.array1: empty text, error; CORREL.array2: empty text, error, numeric text.
- PEARSON.array1: empty text, error; PEARSON.array2: empty text, error, zero.
- FORECAST and FORECAST.LINEAR known_y's, known_x's: empty text, error.
- COVAR.array1/array2, COVARIANCE.P.array1/array2, COVARIANCE.S.array2: empty text, error;
  COVARIANCE.S.array1: empty text, error, zero.
- INTERCEPT, RSQ, STEYX known_y's/known_x's: empty text, error; SLOPE.known_y's: empty text, error;
  SLOPE.known_x's: empty text, error, numeric text.
- LINEST and LOGEST known_y's/known_x's: empty text, error.
- TREND and GROWTH known_y's/known_x's: empty text, error; GROWTH.new_x's: empty text, error, numeric text;
  TREND.new_x's: a boolean, empty text, error, numeric text.
- PERCENTRANK.array, PERCENTRANK.EXC.array, PERCENTRANK.INC.array, TRIMMEAN.array: empty text, error.
- FREQUENCY.data_array/bins_array, PROB.x_range/prob_range: empty text, error.
- F.TEST and FTEST array1/array2: empty text, error.
- CHITEST.actual_range/expected_range: empty text, error; CHISQ.TEST.actual_range/expected_range: empty text, error,
  numeric text.
- T.TEST.array1/array2: empty text; TTEST.array1: empty text, error; TTEST.array2: empty text, error, numeric text.
- Z.TEST.array, ZTEST.array: empty text, error, numeric text, zero.

Write one probe case, probes/m9-stat-array-argument-kind-gaps (excel-probe skill, excel-graph, tag M9). A short
generator script beside the existing probe generators is fine for a list this long. Put each kind in two forms: a
literal (`""`, `#N/A`, `#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell
`=""`, `=NA()` or `=1/0`, or a typed `TRUE`, `"2"` or `0`). Give the other arguments ordinary valid values (a 4-cell
numeric range for the other array, k = 1, p = 0.5 and so on), so the probed argument decides the result. Wrap each
error-valued cell in `ERROR.TYPE` too, so the error kind is scored. One cell per kind and form for each parameter is
enough.

## Acceptance criteria
- [ ] probes/m9-stat-array-argument-kind-gaps is committed with oracle values (excel-graph) and covers every
      parameter/kind pair listed above. After `make eval`, report.json `argument_coverage` shows `covered == expected`
      for the listed parameters, or the Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-stat task (`campaign: semantics`, with a `## Visible-parity stage` section)
      filed with `make new-task`, naming the case and cells. The new known failures are recorded with
      `make eval-baseline` and each has that owner (`make unowned-off ALL=1` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/statistics.md, citing the probe cells. One example is
      whether a scalar `""` or error passed as a data array gives an error, counts as a one-element sample or is
      ignored.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The FORECAST.ETS family's `values` and `timeline` (desktop-only numbers; T-2425, T-1553).
- Value-mode parameters of these functions (T-3129) and of the distribution functions (T-3127, T-3128).
- Non-statistical M9 functions (financial: T-3184 and T-3125).
