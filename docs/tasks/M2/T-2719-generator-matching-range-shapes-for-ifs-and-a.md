---
id: T-2719
title: "Generator: matching range shapes for *IFS and a vector MATCH lookup_array"
milestone: M2
lane: harness
status: done
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
Generated SUMIFS/COUNTIFS (and AVERAGEIFS/MAXIFS/MINIFS) formulas mostly compute a real criteria match instead of
the shape-mismatch `#VALUE!`, and generated MATCH formulas mostly search a 1-D lookup_array, so "covered" in the
fifteen-digit checkpoint means the matching itself is exercised.

## Context
- Found by T-2673's Excel-fidelity review. Over T-2673's 9 new gen-m2 cases (seed 2673), Excel's values for the
  outer-call cells are errors in SUMIFS 154/156 (all `#VALUE!`), COUNTIFS 102/154 (all `#VALUE!`) and MATCH 136/157
  (112 `#N/A`, 17 `#VALUE!`, 5 `#REF!`); COUNTIF and SUMIF have 0 errors. Examples: gen-m2/b9c807444a35c9a2 Calc!A114
  `=SUMIFS(Data!C2:C9,Data!A10:D11,"?b?")`, A137 `=SUMIFS(Data!A3:D6,Data!A6:B10,"<>")`.
- Cause: `argument_text` in `tools/src/rustsheet_tools/gen/generator.py` picks the shape of each `range` parameter
  independently, and MATCH's `lookup_array` in `eval/catalog/functions.yaml` lacks the `vector: true` that
  LOOKUP's `lookup_vector` has (the flag always gives a one-column range, so the 2-D rate needs its own knob).

## Acceptance criteria
- [x] Within one *IFS call, sum_range/average_range/max_range and every criteria_range share one shape (rows x
      columns), with a small deliberate-mismatch rate (about 1 in 10) so the `#VALUE!` path stays covered.
- [x] MATCH's lookup_array is a 1-D range (a row or a column), with a small 2-D rate kept for the `#N/A` path.
- [x] Generator unit tests cover both rules.
- [x] A top-up batch, `rustsheet-gen --milestone M2 --focus --functions SUMIFS,COUNTIFS,MATCH --count 2 --formulas 300
      --seed 2719`, is generated, excel-graph filled and baselined (0 regressions). Fewer than 25% of its outer-call
      cells are errors for each of the three functions. Any new mismatch family gets a functions task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Text-number criteria in the criteria pool (T-2717).
- Rewriting existing case files (they are stored; only new cases change).

## Outcome
- `same_shape` (catalog) gives the sum/average/max/min range and every criteria range of one *IFS call a shared
  rows x columns shape; about 1 call in 10 stays free so the `#VALUE!` mismatch remains covered.
- MATCH: `lookup_array` is `vector` (mostly a column, 20% a row, 5% 2-D); `lookup_value` is `probe`d from the
  lookup range's cells (95%); `match_type` is always passed, mostly 0. Together these were needed to get MATCH under
  the error bar: the first attempt (shapes only) left 71% errors, mostly `#N/A` from lookup values that were not in the data.
- Top-up batch (`--milestone M2 --focus --functions SUMIFS,COUNTIFS,MATCH --count 2 --formulas 300 --seed 2719`):
  gen-m2/593331b2d8c3cce7 and gen-m2/940f02dcde28721f, excel-graph filled. Outer-call errors: SUMIFS 25/200,
  COUNTIFS 13/200, MATCH 49/200 (24.5%). No mismatch families beyond the baselined cases (baseline +2 case lines).
- Eval delta: no regressions; two new cases added.
