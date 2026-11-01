---
id: T-3799
title: "Gate A: classify the 74 M9 sample-statistics functions (descriptive, regression, rank, test and criteria aggregates) with no table or exemption, and file their table tasks"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every in-scope numeric M9 function that computes a statistic over sample data and that Gate A counts as missing ends
up in one of two states:
- it has an `eval/accuracy/exempt.tsv` line backed by a decision record;
- or an open task names it in its title or Goal and will build its table.

## Context
- Gate A holds only with no missing table (SPEC-0029 "Gate A holds", decision 0188 R1(a)). Decision 0228 classified
  M5, M6 and M7. Its Consequences, and T-3551's Out of scope, leave M9's functions outside decision 0188's 18 for a
  separate classification task. These functions take arrays, which the generator writes as one-row array literals
  (T-3586, done). 2-D literals are T-3686 (open).
- `rustsheet-eval accuracy` on main (56917e82c) prints `missing tables 271`. 74 of them are M9 statistical functions
  over sample data that no Gate A task names:
  AVEDEV, AVERAGEA, AVERAGEIF, AVERAGEIFS, CHISQ.TEST, CHITEST, CORREL, COVAR, COVARIANCE.P, COVARIANCE.S, DEVSQ,
  F.TEST, FORECAST, FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT, FORECAST.LINEAR,
  FREQUENCY, FTEST, GEOMEAN, GROWTH, HARMEAN, INTERCEPT, KURT, LARGE, LINEST, LOGEST, MAXA, MAXIFS, MEDIAN, MINA,
  MINIFS, MODE, MODE.MULT, MODE.SNGL, PEARSON, PERCENTILE, PERCENTILE.EXC, PERCENTILE.INC, PERCENTRANK,
  PERCENTRANK.EXC, PERCENTRANK.INC, PROB, QUARTILE, QUARTILE.EXC, QUARTILE.INC, RANK, RANK.AVG, RANK.EQ, RSQ, SKEW,
  SKEW.P, SLOPE, SMALL, STDEV, STDEV.P, STDEV.S, STDEVA, STDEVP, STDEVPA, STEYX, T.TEST, TREND, TRIMMEAN, TTEST, VAR,
  VAR.P, VAR.S, VARA, VARP, VARPA, Z.TEST, ZTEST.
  T-3693 mentions LARGE, SMALL, PERCENTILE.* and QUARTILE.* only as AGGREGATE 14 to 19, so it builds no table for
  them. Recount before filing, because tasks merged in the meantime may change the list.
- Questions that need judgment, each answered in the decision record:
  - **Reference-only arguments.** statistics.md "RANK of a non-reference array": RANK, RANK.EQ and RANK.AVG need a
    reference, and excel-graph refuses a workbook with `RANK(5,{7,1,5})`. So no literal table can be filled, and they
    are `composite`, like SUBTOTAL in decision 0228.
  - **Criteria aggregates** (AVERAGEIF, AVERAGEIFS, MAXIFS, MINIFS). They take ranges. Classify them like SUMIF and
    SUMIFS (`composite`, decision 0217, T-3550), unless a criteria form takes an array literal.
  - **Selection without arithmetic** (MAXA, MINA, LARGE, SMALL, MODE, MODE.SNGL, MODE.MULT, FREQUENCY). Each is
    `exact` only if its result is always one of its inputs or a count, with no floating-point step. MEDIAN is not
    (statistics.md "MEDIAN of an even count is a + (b - a)/2"). MODE.MULT and FREQUENCY return arrays: note how a
    one-cell point reads them, or classify them `composite`.
  - **The FORECAST.ETS family.** statistics.md "FORECAST.ETS family: excel-graph never computes it": every valid call
    is `#N/A` on the web oracle, and T-2425 (M14, blocked) owns the implementation. A table filled from excel-graph
    holds no number. Decide `composite` (naming the gen-m9 and probes cases) or a table blocked on T-2425's desktop
    evidence, and record which.
  - **Array results** (LINEST, LOGEST, TREND, GROWTH). They need 2-D literals for a multi-column x, so their table
    tasks depend on T-3686. Note how the point reads one element (for example through INDEX), or classify them
    `composite`.
  - **Variadic aggregates** (STDEV*, VAR*, DEVSQ, AVEDEV, KURT, SKEW*, GEOMEAN, HARMEAN, AVERAGEA, TRIMMEAN). They
    need a table. The draw of a variable-length array and the near-zero rule follow T-3549's SUM and AVERAGE
    definitions, so these table tasks depend on T-3549.
  - **Tests** (T.TEST, TTEST, F.TEST, FTEST, CHISQ.TEST, CHITEST, Z.TEST, ZTEST, PROB). They need a table over paired
    arrays, reusing the distributions' definitions (T-3358, T-3553, and T-3529 for CHISQ).
- `exempt.tsv` is protected (decision 0199, T-3379): every line added needs the decision record. The three sibling
  classification tasks (T-3797, T-3798, T-3800) also add lines, so rebase before merging and keep the file sorted.
- Group the functions that need a table into table tasks of at most about seven functions each (M tasks, like
  T-3668 to T-3678), by kernel: dispersion, shape, means, order statistics and percentiles, regression, tests.
  Create each with `make new-task MILESTONE=M0 LANE=harness`, `tier: opus`, `spec:` SPEC-0029.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) classifies each of the 74 functions (or the recounted list) as
      `exact`, `composite` or "needs a table (T-XXXX)", with a cited reason for each exemption, including the
      FORECAST.ETS ruling.
- [ ] `eval/accuracy/exempt.tsv` gains the exempt lines, sorted. `rustsheet-eval accuracy` reports `missing tables`
      lower by their number, with no stale or bad exemption.
- [ ] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR, which
      depends on T-3549, T-3686 or T-3529 where the notes above say so.
- [ ] `python3 scripts/tasks.py validate` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building any table, definition or generator change (the filed tasks do that).
- AGGREGATE (T-3693), SUM and AVERAGE (T-3549), and the M2 and M5 aggregates (decision 0228, T-3550).
- The other M9 families: financial (T-3797), distributions (T-3798), engineering and database (T-3800).
- Implementing the FORECAST.ETS family (T-2425).
