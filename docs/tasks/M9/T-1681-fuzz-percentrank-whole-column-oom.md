---
id: T-1681
title: "Fuzz: PERCENTRANK.INC over SE:RCH aborts with a 3 GB allocation: bound values_from_range on whole-column ranges"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A formula whose range argument covers whole columns of an almost empty sheet (`SE:RCH`, 7,000+
columns × 1,048,576 rows) never aborts the process by allocation (directive 5). The
order-statistics and paired-list functions give Excel's result in bounded memory.

## Context
Nightly fuzz run 38847367745 (2026-10-16, `main` 1aebd11c), target `formula_eval`, artifact
`oom-628770d99ec69d8b480f8269cc4c0d84b3ca8105`. The input is the 33 bytes
`PERCENTRANK.INC(SE:RCH,"İİx",2)` (the text holds two U+0130; base64
`UEVSQ0VOVFJBTksuSU5DKFNFOlJDSCwixLDEsHgiLDIp`). The target sets it as `=...` in `C3` of its small
model (`fuzz/fuzz_targets/formula_eval.rs`).

The steward reproduced it on a18a7329 with a release-build facade test (`Workbook::set("C3", ...)`,
`ulimit -v 4000000`): `memory allocation of 3019898896 bytes failed`, then `rust_oom` aborts.
The text argument has nothing to do with it:
- `PERCENTRANK.INC(SE:RCH,2)`, `PERCENTILE.INC(SE:RCH,0.5)` and `QUARTILE(SE:RCH,1)` abort the same way.
- `MEDIAN(SE:RCH)` and `SUM(SE:RCH)` return in 4 ms, because they use the clipping walker.

Root cause: `collect_sorted_values` (`functions/statistical/percentile.rs`) calls
`values_from_range` (`functions/statistical/chisq.rs:262`). On one sheet, that function builds one
`Option<f64>` per cell of the range on purpose, unclipped, because paired callers (CHISQ.TEST,
SUMXMY2, COVARIANCE and others) match two lists by position. Other callers of `values_from_range`
may have the same exposure: `mode_functions.rs`, `fisher.rs`, `t_dist.rs`, `frequency.rs`,
`prob.rs`, `z_test.rs`, `covariance.rs` and `math_and_trigonometry/sum.rs`.

Fix direction:
- Callers that only need the numbers, like the percentile and quartile family, go through the
  existing clipping walk (`for_each_cell_or_error_in_areas`, as MEDIAN does).
- Paired callers clip both ranges to the same used-area window, or read blanks past the used area
  without allocating them.
- Search for an existing helper first (`rg`). Never add a second copy.

Excel's values for whole-column order statistics: check probes for existing evidence. If none
exists, write a probe case (`excel-probe` skill).

## Acceptance criteria
- [x] The reproducer is a unit test (decision 0035). It passes with no huge allocation, for example
      by asserting it returns quickly, or by running under a bounded allocator.
- [x] Every caller of `values_from_range` is bounded by the used area on whole-column and
      whole-row ranges. Each caller kind gets a test: order statistics, paired lists, and
      MODE/FREQUENCY.
- [x] The values match excel-graph evidence. Otherwise the case is left pending and the task is
      blocked.
- [x] `make check` green; reviewers approve.

## Out of scope
- Performance work on bounded ranges beyond removing the per-cell allocation (perf lane).

## Outcome
- `values_from_range` now walks the used area (the clipping walk SUM and MEDIAN use), so the
  order statistics (PERCENTILE, PERCENTRANK, QUARTILE, RANK), MODE, FREQUENCY, PROB, Z.TEST and
  F.TEST hold one entry per used-area cell, not per cell of the range.
- Callers that pair two lists by position use one new helper, `paired_values`
  (`functions/statistical/paired.rs`). These are SUMXMY2, SUMX2MY2 and SUMX2PY2; PEARSON, RSQ,
  CORREL, SLOPE, INTERCEPT, STEYX and FORECAST through `fn_get_two_matrices`; COVARIANCE.P and
  COVARIANCE.S; and paired T.TEST.
  - The helper sizes each argument from its shape, so a size mismatch is still `#N/A`.
  - Both lists stop where the data of both arguments stops: past the used area every pair is
    blank, and every paired caller skips a pair with a blank side.
  - It reads every used cell, so an error past the paired positions still comes first. The first
    argument's errors come before the second's.
  - CHISQ.TEST takes its "two data points" check from the shape, not the list length.
- Shape arithmetic is `i64`, because 16,384 × 1,048,576 overflows `i32`.
- The duplicated per-argument code of COVARIANCE.P/S, T.TEST and `fn_get_two_matrices` is gone.
  Duplicated lines went from 6,328 to 6,155, and the ratchet can tighten.
- Evidence: no Excel behavior changes, and no new behavior is claimed. Each whole-column result
  equals the result over the used area, which these functions already returned with full lists.
  That is because blank cells in a range are ignored, as for SUM over whole columns (T-0048,
  `test_clipped_range_walks`). The new tests check that equivalence for every caller kind. They
  also cover the fuzz reproducer, `PERCENTRANK.INC(SE:RCH,"İİx",2)`, with a time bound.
  CHISQ.TEST keeps counting every cell of a whole column in its degrees of freedom, as before;
  that is unprobed.
- Eval delta: 0 fixed, 0 regressions in every gated suite.
