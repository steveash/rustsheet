---
id: T-3129
title: "Probe the M9 aggregate, test, regression, conditional-aggregate and database argument kinds no Excel cell covers yet (48 statistical functions, 12 D-functions)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value or criteria parameter of the M9 aggregate, test, regression and conditional-aggregate functions below, and the `field` parameter of the 12 D-functions, has at least one Excel-backed cell for each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-29 (`make eval` on 6793e919 with calipers, corpus-bench and poi fetched; report.json
`argument_coverage`, decision 0031 evidence gaps; decision 0094 asks for each argument mode to be covered). Every
visibly-off cell of that run has an open owner (`scripts/exit_owners.py --all-off`), and the evidence floor is met, so
the M9 argument kinds are the largest evidence gap left: 245 M9 functions have a value parameter with a kind no Excel
cell passes (M2 was closed by T-2861; M5 to M7 are T-2862, T-2863, T-2864, T-2865 and T-2871 to T-2878; M9 had none).
The kinds are those of `crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative,
fraction, numeric_text, text, empty_text, boolean, error, blank, range. A kind is seen only when an Excel-evidence cell
passes it as a literal or a single-cell reference: nested calls such as `NA()` are not classified (docs/progress.md
"Argument coverage"), so use literal errors (`#N/A`, `#DIV/0!`) and cells holding them. T-2866 taught the generator to
draw `""` and error literals, but at 3% per parameter the existing gen-m9 cases predate it and do not reach these pairs.
`array`-mode parameters are left out: their coverage is not classified by kind.
Three kinds of gap: empty text or an error as a direct argument of an aggregate (MEDIAN, STDEV, VAR, MODE, SKEW, KURT, ...) or as k/quart/significance/tails/type; a zero, negative, fraction, numeric text, boolean or error criteria in AVERAGEIF(S), MAXIFS and MINIFS (criteria.md rules apply; put the same value in the range too, as T-2861 did for SUMIF); and an error `field` in DAVERAGE...DVARP (criteria.md covers the D-functions). LINEST, LOGEST, TREND and GROWTH const/stats need an error, a zero and a multi-cell range.
These parameters have no Excel cell of the listed kinds in any suite (highest catalog `prior` first, as
`FUNCTION.parameter: kinds`):
- AVERAGEIF.criteria: zero, a fraction, numeric text, a boolean.
- AVERAGEIFS.criteria: zero, a negative, a fraction, numeric text, a boolean, an error.
- LARGE.k: empty text, an error.
- MEDIAN.number: empty text.
- RANK.number: empty text; RANK.order: empty text.
- RANK.AVG.number: empty text; RANK.AVG.order: empty text.
- RANK.EQ.number: empty text, an error; RANK.EQ.order: empty text, an error, a multi-cell range.
- SMALL.k: empty text, an error.
- STDEV.number: empty text.
- STDEV.S.number: empty text, an error.
- FORECAST.x: an error.
- FORECAST.LINEAR.x: an error.
- MAXIFS.criteria: zero, a negative, a fraction, numeric text, a boolean, an error.
- MINIFS.criteria: zero, a negative, a fraction, numeric text, a boolean, an error.
- PERCENTILE.k: empty text.
- PERCENTILE.EXC.k: empty text, an error.
- PERCENTILE.INC.k: empty text, an error.
- QUARTILE.quart: empty text.
- QUARTILE.EXC.quart: empty text.
- QUARTILE.INC.quart: empty text.
- STDEV.P.number: empty text.
- STDEVP.number: empty text, an error.
- VAR.number: empty text.
- VAR.P.number: empty text.
- VAR.S.number: empty text, an error.
- VARP.number: empty text, an error.
- AVEDEV.number: empty text.
- DEVSQ.number: empty text.
- GROWTH.const: an error, a multi-cell range.
- HARMEAN.number: empty text.
- KURT.number: empty text.
- LINEST.const: an error, a multi-cell range; LINEST.stats: zero, an error, a multi-cell range.
- LOGEST.const: an error, a multi-cell range; LOGEST.stats: an error, a blank cell, a multi-cell range.
- MODE.number: empty text.
- MODE.MULT.number: empty text.
- MODE.SNGL.number: empty text.
- PERCENTRANK.x: empty text; PERCENTRANK.significance: empty text, a boolean, an error.
- PERCENTRANK.EXC.x: empty text; PERCENTRANK.EXC.significance: numeric text, empty text, an error.
- PERCENTRANK.INC.x: empty text; PERCENTRANK.INC.significance: empty text, an error.
- SKEW.number: empty text.
- SKEW.P.number: empty text, an error.
- TREND.const: an error, a multi-cell range.
- VARPA.value: empty text.
- T.TEST.tails: empty text, an error; T.TEST.type: empty text, an error.
- TRIMMEAN.percent: empty text, an error.
- TTEST.tails: empty text, an error; TTEST.type: empty text, an error.
- Z.TEST.x: empty text, an error; Z.TEST.sigma: empty text, an error.
- ZTEST.x: empty text, an error; ZTEST.sigma: empty text, an error.
- DCOUNT.field: an error.
- DGET.field: an error.
- DSUM.field: an error.
- DAVERAGE.field: an error.
- DCOUNTA.field: empty text, an error.
- DMAX.field: an error.
- DMIN.field: an error.
- DPRODUCT.field: an error.
- DSTDEV.field: an error.
- DSTDEVP.field: an error.
- DVAR.field: an error.
- DVARP.field: an error.
Write the probe case probes/m9-stat-aggregate-argument-kind-gaps (excel-probe skill, excel-graph, tag M9); a short generator script beside the existing
probe generators is fine when the list is long. Put each kind in two forms where they differ: a literal (`""`, `#N/A`,
`#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell `=""`, `=1/0` or `=NA()`, a typed `TRUE`
or `"2"`, an empty cell for the blank kind). Give every other
argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued cell in
`ERROR.TYPE` too so the error kind is scored.

## Acceptance criteria
- [ ] probes/m9-stat-aggregate-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed above; after
      `make eval`, report.json `argument_coverage` shows `covered == expected` for the listed parameters, or the
      Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (`campaign: semantics`, with a `## Visible-parity stage` section) filed
      with `make new-task`, naming the case and cells; the new known failures are recorded with `make eval-baseline` and
      each has that owner (`make unowned-off` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/statistics.md or criteria.md, citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Distribution functions (T-3127, T-3128); FORECAST.ETS and its options (T-2371, T-2425).
- `array`-mode parameters (CORREL, PEARSON, COVAR and the other paired-range functions).
- Pairwise argument orders (which of two bad arguments wins): T-2611 and the check-order sections of financial.md own
  those for the financial functions; file others as follow-ups.
- Generator pools (T-2866, done).
