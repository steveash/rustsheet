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
- Memory is now proportional to the non-blank cells, never to the cells of a range or of its used
  area. A far cell can stretch the used area, so a whole column is walked over that area, which
  costs time as SUM does, but a blank cell gets no entry.
- `values_from_range` (via `values_from_areas`) skips blank cells. Its callers keep only the
  numbers: the order statistics (PERCENTILE, PERCENTRANK, QUARTILE, RANK), MODE, FREQUENCY,
  PROB, Z.TEST, F.TEST and unpaired T.TEST.
- Callers that pair two lists by position use one new helper, `paired_values`
  (`functions/statistical/paired.rs`). These are SUMXMY2, SUMX2MY2 and SUMX2PY2; PEARSON, RSQ,
  CORREL, SLOPE, INTERCEPT, STEYX and FORECAST through `fn_get_two_matrices`; COVARIANCE.P and
  COVARIANCE.S; and T.TEST.
  - Each argument becomes (row-major position, value) entries for its non-blank cells.
  - The two are merge-joined into aligned lists that leave out the positions blank on both sides,
    which every paired caller treats as no data.
  - Sizes come from the shapes, so a mismatch is still `#N/A`, after every cell error.
  - The first argument is read in full before the second, so its errors come first.
  - `paired` counts the pairs that zipping full lists made, for paired T.TEST on unequal sizes.
    Unpaired T.TEST reads each argument once.
  - CHISQ.TEST takes its "two data points" check from the shape.
- Shape arithmetic is `i64`, because 16,384 × 1,048,576 overflows `i32`.
- The duplicated per-argument code of COVARIANCE.P/S, T.TEST and `fn_get_two_matrices` is gone.
  Duplicated lines went from 6,328 to 6,155, and the ratchet can tighten.
- Evidence: no Excel behavior changes, and no new behavior is claimed. Each whole-column or
  whole-row result equals the result over the used area, which these functions already returned
  with full lists, because blank cells in a range are ignored (as SUM over whole columns, T-0048).
  The new tests in `test_clipped_range_walks` check that equivalence for every caller kind. They
  also check a far cell (`CV5000`), whole rows, a size mismatch, a late error, and the fuzz
  reproducer `PERCENTRANK.INC(SE:RCH,"İİx",2)`, with time bounds.
- CHISQ.TEST's degrees of freedom over a whole column count every cell, as before. That is
  unprobed; T-1689 probes it.
- Messages: COVARIANCE.P and COVARIANCE.S share one size-mismatch message. The error is the same.
- Eval delta: 0 fixed, 0 regressions in every gated suite.
