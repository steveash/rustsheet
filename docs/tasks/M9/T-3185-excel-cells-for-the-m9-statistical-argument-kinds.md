---
id: T-3185
title: "Excel cells for the M9 statistical argument kinds no suite covers (573 parameter/kind pairs in 129 functions, mostly empty text and errors)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every parameter of the 129 M9 statistical functions below has an Excel-backed cell of each argument kind that
report.json `argument_coverage` expects (decision 0094: coverage "across the argument modes its catalog signature
lists"). Every new cell that differs from Excel gets an owner.

## Context
Gap triage 2026-10-30 (`make eval` on main c269641be; decision 0031 evidence gaps). The M2 to M7 argument-kind gaps
have probe tasks (T-2861 to T-2878). M9 has none. Of M9's 1,392 missing parameter/kind pairs, the statistical
category has 662 in 133 functions. Leaving out the FORECAST.ETS family, which is not implemented and waits for desktop
evidence (T-2425, T-1553), leaves 573 pairs in 129 functions. By kind (`classify_value` in
crates/rustsheet-eval/src/argument_coverage.rs): empty_text 285, error 211, numeric_text 35, boolean 12, zero 9,
range 7, text 6, fraction 4, negative 3, blank 1. The full list is report.json `argument_coverage` filtered to catalog
`category: statistical`, `milestone: M9`, `covered < expected`.

Shapes that need care:
- Distribution scalars, for example NORM.DIST x, mean, standard_dev and cumulative; BETA.DIST, BINOM.DIST, GAMMA.DIST,
  T.DIST and the legacy names. Most lack only `""` and an error.
- Array and aggregate parameters, which take a range or array whole: FREQUENCY data_array and bins_array; LARGE and
  SMALL array; PROB x_range and prob_range; T.TEST array1 and array2; CHISQ.TEST and CHITEST actual_range and
  expected_range (numeric text too); LINEST, LOGEST, GROWTH and TREND known_y's, known_x's and new_x's. Excel's rule for a
  `""` or an error inside an array argument is per function (docs/excel-semantics/statistics.md, arrays.md). So each
  needs the kind both as a direct argument and as an array element.
- Criteria parameters of AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS: a boolean, a fraction, numeric text, zero, a negative
  number and (AVERAGEIFS) an error as criteria. Any fix goes to the functions-criteria lane.

The design question is the same as T-3184's: T-2866's `""` and error draws seldom survive `domain_value` and the edge
pools, so new `--focus` batches alone will not reach these kinds. Choose a coverage-directed generator mode,
hand-written excel-probe cases in the T-2861 style (probes/m2-argument-kind-gaps*), or a mix. Record the choice in
docs/architecture/synthetic-generator.md, and share it with T-3184 if that task has landed. Committed gen-m9 cases stay
unchanged. New cases are oracle-filled with excel-graph.

## Acceptance criteria
- [ ] After `make eval`, report.json `argument_coverage` shows `covered == expected` for every parameter of the 129
      functions. Where Excel refuses a workbook holding a kind, the kind is exempted in the catalog with its reason
      instead.
- [ ] Every new cell where ours differs from Excel is fixed in this PR, if it is a one-line coercion fix in the
      function's own module, or listed by case and cell in a functions-stat or functions-criteria task filed with
      `make new-task` (`campaign: semantics` and a `## Visible-parity stage` section for wrong types and errors). The
      new known failures are recorded with `make eval-baseline`, and `make unowned-off ALL=1` lists none of them.
- [ ] Each new Excel rule is in docs/excel-semantics/statistics.md (or criteria.md, arrays.md), citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Covering every function in one PR: if the work exceeds size M, split it by function group and file the rest
  as follow-up tasks with `make new-task` (as T-2861..T-2878 did).
- FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT (T-2425).
- The financial (T-3184), engineering and database argument kinds.
- Fixing numeric residue of new cells (file it as tasks).
